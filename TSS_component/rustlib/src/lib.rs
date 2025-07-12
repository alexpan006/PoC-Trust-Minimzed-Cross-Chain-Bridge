pub mod bitcoin_related;
use once_cell::sync::Lazy;
use pyo3::prelude::*;
use rand::rngs::OsRng;
use sled::{Db};
use std::collections::BTreeMap;
use frost_secp256k1_tr::{
    keys::{dkg, KeyPackage, PublicKeyPackage, Tweak}, Error as FrostError, Identifier, Signature, SigningPackage
};
use frost_secp256k1_tr::round1;
use frost_secp256k1_tr::round2;
use frost_secp256k1_tr::round2::SignatureShare;
use frost_secp256k1_tr::keys::dkg::{round1 as dkgRound1,round2 as dkgRound2};
use thiserror::Error; // For better error handling
static DB: Lazy<Db> = Lazy::new(|| sled::open("/state/nonces_db").expect("sled"));
// static mut KEYPKG: Option<Zeroizing<KeyPackage>> = None;       // safe via GIL

// --- Error Handling (adapted for u16 context) ---
#[derive(Error, Debug)]
enum FfiError {
    #[error("sled DB error: {0}")]
    Sled(#[from] sled::Error),
    #[error("FROST error: {0}")]
    Frost(FrostError<>),
    #[error("Hex decoding error: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("Serialization/Deserialization error: {0}")]
    Serde(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid Identifier from u16: {0}")] // Specific error if TryFrom<u16> fails
    InvalidIdentifierU16(String),
    #[error("State error: {0}")]
    State(String),
    #[error("Missing required data for ID {0}")]
    MissingData(String), // Use u16 for missing data ID
}

impl From<FfiError> for PyErr {
    fn from(err: FfiError) -> PyErr {
        match err {
            FfiError::Sled(e) => pyo3::exceptions::PyConnectionError::new_err(format!("Database error: {}", e)),
            FfiError::Frost(e) => pyo3::exceptions::PyValueError::new_err(format!("FROST protocol error: {}", e)),
            FfiError::Hex(e) => pyo3::exceptions::PyValueError::new_err(format!("Invalid hex string: {}", e)),
            FfiError::Serde(e) => pyo3::exceptions::PyValueError::new_err(format!("Serialization error: {}", e)),
            FfiError::Json(e) => pyo3::exceptions::PyValueError::new_err(format!("JSON error: {}", e)),
            FfiError::InvalidIdentifierU16(e) => pyo3::exceptions::PyValueError::new_err(format!("Identifier error: {}", e)),
            FfiError::State(e) => pyo3::exceptions::PyRuntimeError::new_err(e),
            FfiError::MissingData(id) => pyo3::exceptions::PyRuntimeError::new_err(format!("Missing data for ID {}", id)),
        }
    }
}

// Specific From implementation for FrostError
impl From<FrostError> for FfiError {
     fn from(e: FrostError) -> Self {
         FfiError::Frost(e)
     }
}

// Helper function to convert Identifier back to u16 (Keep as provided by user)
// WARNING: This is potentially lossy and relies on specific library behavior.
fn identifier_to_hex(id: Identifier) -> String {
    let id_hex = hex::encode(id.serialize());
    id_hex
}
fn identifier_from_hex(id: String) -> Identifier {
    let id_identifier =Identifier::deserialize(&hex::decode(&id).unwrap()).unwrap();
    id_identifier
}

// Helper to convert u16 to Identifier using TryFrom
fn identifier_from_u16(id_u16: u16) -> Result<Identifier, FfiError> {
     Identifier::try_from(id_u16)
          .map_err(|e| FfiError::InvalidIdentifierU16(format!("Failed to convert u16 {} to Identifier: {:?}", id_u16, e)))
}

// ------------------ DKG round‑1 -----------------------------------
#[pyfunction]
fn dkg_round1(self_id: u16, total_signers: u16, min_signers: u16) -> PyResult<(String,String)> { // Return only broadcast pkg hex
    let id = identifier_from_u16(self_id)?;
    let id_hex = hex::encode(id.serialize());
    let (secret1, pkg1) = dkg::part1(id, total_signers, min_signers, &mut OsRng).unwrap(); // Propagate FROST errors

    // persist my round‑1 secret package for later rounds
    let secret1_bytes = secret1.serialize().map_err(|e| FfiError::Serde(format!("Serialize r1 secret failed: {}", e)))?;
    DB.insert(format!("r1_{}", id_hex), secret1_bytes).unwrap(); // Propagate sled errors

    let pkg1_bytes = pkg1.serialize().map_err(|e| FfiError::Serde(format!("Serialize r1 pkg failed: {}", e)))?;
    Ok((id_hex,hex::encode(pkg1_bytes))) // Return broadcast pkg hex
}

// ------------------ DKG round‑2 -----------------------------------
#[pyfunction]
fn dkg_round2(self_id: String,
              received_pkgs_hex: Vec<(String, String)>) -> PyResult<String> { // Return broadcast JSON string only
    // load my secret‑1 package
    let secret1_key = format!("r1_{}", self_id);
    // println!("[Rust] Loading secret1 from sled DB with key: {}", secret1_key);
    let secret1_bytes = DB.get(&secret1_key).unwrap()
                          .ok_or_else(|| FfiError::MissingData(self_id.clone()))?; // Use MissingData error
    let secret1: dkgRound1::SecretPackage = dkgRound1::SecretPackage::deserialize(&secret1_bytes)
                                                .map_err(|e| FfiError::Serde(format!("Deserialize r1 secret failed: {}", e)))?;

    // reconstruct map<Identifier, Package>
    let mut map = BTreeMap::new();
    for (peer_id, hex_pkg) in received_pkgs_hex {
        let peer_ident =Identifier::deserialize(&hex::decode(&peer_id).unwrap()).unwrap();
        let pkg_bytes = hex::decode(hex_pkg).unwrap();
        let pkg: dkgRound1::Package = dkgRound1::Package::deserialize(&pkg_bytes)
                                            .map_err(|e| FfiError::Serde(format!("Deserialize r1 pkg from {} failed: {}", peer_id, e)))?;
        map.insert(peer_ident, pkg);
    }

    let (secret2, pkgs2) = dkg::part2(secret1, &map).map_err(FfiError::Frost)?;

    // Persist secret2
    let secret2_bytes = secret2.serialize().map_err(|e| FfiError::Serde(format!("Serialize r2 secret failed: {}", e)))?;
    DB.insert(format!("r2_{}", self_id), secret2_bytes).unwrap();

    // convert map back to Vec<(u16, String)> for easy JSON
    let out: Vec<(String, String)> = pkgs2.into_iter()
        .map(|(id, p)| {
            let u16_id = hex::encode(id.serialize()); 
            let pkg_bytes = p.serialize().map_err(|e| FfiError::Serde(format!("Serialize r2 pkg for {} failed: {}", u16_id, e)))?;
            Ok((u16_id, hex::encode(pkg_bytes)))
        })
        .collect::<Result<Vec<_>, FfiError>>()?;

    // Return broadcast JSON string
    Ok(serde_json::to_string(&out).unwrap())
}

// ------------------ DKG round‑3 -----------------------------------
#[pyfunction]
fn dkg_round3(
    self_id: String,
    r1_pkgs_hex: Vec<(String, String)>,
    r2_pkgs_hex: Vec<(String, String)>,
) -> PyResult<(String, String)> { // (KeyPackage hex, PublicKeyPackage hex, VerifyKey hex)
    // Load secret2
    let secret2_key = format!("r2_{}", self_id);
    let secret2_bytes = DB.get(&secret2_key).unwrap()
                          .ok_or_else(|| FfiError::MissingData(self_id.clone()))?;
    let secret2: dkgRound2::SecretPackage = dkgRound2::SecretPackage::deserialize(&secret2_bytes)
                                                 .map_err(|e| FfiError::Serde(format!("Deserialize r2 secret failed: {}", e)))?;

    // Reconstruct maps
    let mut r1_map: BTreeMap<Identifier, dkg::round1::Package> = BTreeMap::new();
    for (pid, h) in r1_pkgs_hex.iter() {
        let id = Identifier::deserialize(&hex::decode(&pid).unwrap()).unwrap();
        let pkg_bytes = hex::decode(h).unwrap();
        let pkg = dkgRound1::Package::deserialize(&pkg_bytes)
                     .map_err(|e| FfiError::Serde(format!("Deserialize r1 pkg from {} failed: {}", pid, e)))?;
        r1_map.insert(id, pkg);
    }
    let mut r2_map = BTreeMap::new();
    for (pid, h) in r2_pkgs_hex.iter() {
        let id = Identifier::deserialize(&hex::decode(&pid).unwrap()).unwrap();
        let pkg_bytes = hex::decode(h).unwrap();
        let pkg = dkgRound2::Package::deserialize(&pkg_bytes)
                     .map_err(|e| FfiError::Serde(format!("Deserialize r2 pkg from {} failed: {}", pid, e)))?;
        r2_map.insert(id, pkg);
    }

    let (kp, pubkp) = dkg::part3(&secret2, &r1_map, &r2_map).map_err(FfiError::Frost)?;

    // Persist KeyPackage and PublicKeyPackage
    let kp_bytes = kp.serialize().map_err(|e| FfiError::Serde(format!("Serialize KeyPackage failed: {}", e)))?;
    DB.insert(format!("keypkg_{}", self_id), kp_bytes.clone()).unwrap();

    let pubkp_bytes = pubkp.serialize().map_err(|e| FfiError::Serde(format!("Serialize PublicKeyPackage failed: {}", e)))?;
    DB.insert(format!("pubkeypkg_{}", self_id), pubkp_bytes.clone()).unwrap();

    // --- Extract and Serialize the Verifying Key ---
    let group_verify_key = pubkp.verifying_key();
    
    // After getting group_verify_key
    let serialized_verify_key = group_verify_key.serialize().unwrap();
    // Always produce 32-bytes x-only format
    let verify_key_bytes = if serialized_verify_key.len() == 33 && (serialized_verify_key[0] == 0x02 || serialized_verify_key[0] == 0x03) {
        serialized_verify_key[1..].to_vec()  // remove prefix byte
    } else if serialized_verify_key.len() == 32 {
        serialized_verify_key.to_vec()        // already fine
    } else {
        return Err(FfiError::State(format!("Unexpected verifying key length: {}", serialized_verify_key.len()).into()).into());
    };

    // Now safe 32-bytes
    let verify_key_hex = hex::encode(verify_key_bytes);
    // println!("[Rust] Verifying key: {}", verify_key_hex);
    Ok((hex::encode(pubkp_bytes), verify_key_hex))
}


// --- Getters (using u16 ID, fixed unwrap) ---
fn get_key_package(self_id: String) -> PyResult<Option<KeyPackage>> {
    match DB.get(format!("keypkg_{}", self_id)).unwrap() { // Use ?
        Some(bytes) =>{
            let kp: KeyPackage = KeyPackage::deserialize(&bytes)
                .map_err(|e| FfiError::Serde(format!("Deserialize keypkg failed: {}", e)))?;
            Ok(Some(kp))
        } 
        None => Ok(None),
    }
}

fn get_public_key_package(self_id: String) -> PyResult<Option<PublicKeyPackage>> {
     match DB.get(format!("pubkeypkg_{}", self_id)).unwrap() { // Use ?
        Some(bytes) => {
            let pubkp: PublicKeyPackage = PublicKeyPackage::deserialize(&bytes)
                .map_err(|e| FfiError::Serde(format!("Deserialize pubkeypkg failed: {}", e)))?;
            let verifying_key = pubkp.verifying_key();
            let serialized_verify_key = verifying_key.serialize().map_err(|e| FfiError::Serde(format!("Serialize VerifyingKey failed: {}", e)))?;
            let verify_key_hex = hex::encode(serialized_verify_key);
            // println!("[Rust] Verifying key: {}", verify_key_hex);
            Ok(Some(pubkp))
        }
        None => Ok(None),
    }
}

// --- Init (using u16 ID) ---
#[pyfunction]
fn init(self_id: u16) -> PyResult<(bool, String,String,String)> {
    let self_id_ser = Identifier::try_from(self_id)
        .map_err(|_| FfiError::InvalidIdentifierU16(format!("Failed to convert u16 {} to Identifier", self_id)))?
        .serialize();
    let self_id_hex = hex::encode(self_id_ser);
    let kp_key = format!("keypkg_{}", self_id_hex);
    let pubkp_key = format!("pubkeypkg_{}", self_id_hex);

    // Check if both keys exist in the database
    let kp_exists = DB.get(kp_key).unwrap().is_some();
    let pubkp_exists = DB.get(pubkp_key).unwrap().is_some();

    if kp_exists && pubkp_exists {
        let pubkp =get_public_key_package(self_id_hex.clone()).unwrap().unwrap();
            // --- Extract and Serialize the Verifying Key ---
        let group_verify_key = pubkp.verifying_key();
        let serialized_verify_key = group_verify_key.serialize().map_err(|e| FfiError::Serde(format!("Serialize VerifyingKey failed: {}", e)))?;
        let verify_key_hex = hex::encode(serialized_verify_key);
        let pubkp_bytes = pubkp.serialize().map_err(|e| FfiError::Serde(format!("Serialize PublicKeyPackage failed: {}", e)))?;
        let pubkp_hex = hex::encode(pubkp_bytes);


        // println!("[Rust] Keys already exist for participant {}.", self_id);
        Ok((true, verify_key_hex,pubkp_hex,self_id_hex))
    } else {
        // println!("[Rust] No keys found for participant {}.", self_id_hex);
        Ok((false, String::new(),String::new(),self_id_hex)) // Return false and empty string
    }
}



// --- Signing Round 1 ---
#[pyfunction]
fn sign_round1(self_id: String) -> PyResult<String> {
    let key_pkg: KeyPackage = get_key_package(self_id.clone())?
        .ok_or_else(|| FfiError::MissingData(format!("Missing KeyPackage for ID {}", self_id)))?;
    let tweaked_key_pkg = key_pkg.clone().tweak(None::<&[u8]>); // Explicit type needed for None if compiler can't infer

    let (nonces, commitments) = round1::commit(tweaked_key_pkg.signing_share(), &mut rand::thread_rng());

    // Persist nonces
    let nonces_bytes = nonces.serialize().map_err(|e| FfiError::Serde(format!("Serialize SigningNonces failed: {}", e)))?;
    DB.insert(format!("nonces_{}", self_id), nonces_bytes).unwrap();

    let commitments_bytes = commitments.serialize().map_err(|e| FfiError::Serde(format!("Serialize SigningCommitments failed: {}", e)))?;
    Ok(hex::encode(commitments_bytes))
}

// --- Signing Round 2 ---
#[pyfunction]
fn sign_round2(self_id: String, message_hex: String, commitments: Vec<(String, String)>) -> PyResult<String> {
    let key_pkg: KeyPackage = get_key_package(self_id.clone())?
        .ok_or_else(|| FfiError::MissingData(format!("Missing KeyPackage for ID {}", self_id)))?;
    

    let tweaked_key_pkg = key_pkg.clone().tweak(None::<&[u8]>); // Explicit type needed for None if compiler can't infer
    let nonces_bytes = DB.get(format!("nonces_{}", self_id)).unwrap()
        .ok_or_else(|| FfiError::MissingData(format!("Missing nonces for ID {}", self_id)))?;
    let nonces = round1::SigningNonces::deserialize(&nonces_bytes).unwrap();

    let mut commitments_map = BTreeMap::new();
    for (pid_hex, hex_str) in commitments {
        let pid = identifier_from_hex(pid_hex);
        let bytes = hex::decode(hex_str).unwrap();
        let commitment = round1::SigningCommitments::deserialize(&bytes).unwrap();
        commitments_map.insert(pid, commitment);
    }

    let message = hex::decode(message_hex).unwrap();
    let signing_package = SigningPackage::new(commitments_map, &message);

    let sig_share = round2::sign_with_tweak(&signing_package, &nonces, &key_pkg,None).unwrap();
    let serialized = sig_share.serialize();
        
    Ok(hex::encode(serialized))
}

// --- Aggregation (Coordinator) ---
#[pyfunction]
fn aggregate_signature(message_hex: String, sig_shares: Vec<(String, String)>, commitments: Vec<(String, String)>, pubkey_hex: String) -> PyResult<String> {
    let mut sig_map = BTreeMap::new();
    let mut commitments_map = BTreeMap::new();

    for (pid_hex, sig_hex) in sig_shares {
        let pid = identifier_from_hex(pid_hex);
        let bytes = hex::decode(sig_hex).unwrap();
        let sig_share = SignatureShare::deserialize(&bytes).unwrap();
        sig_map.insert(pid, sig_share);
    }

    for (pid_hex, commit_hex) in commitments {
        let pid = identifier_from_hex(pid_hex);
        let bytes = hex::decode(commit_hex).unwrap();
        let commit = round1::SigningCommitments::deserialize(&bytes).unwrap();
        commitments_map.insert(pid, commit);
    }

    let message = hex::decode(message_hex).unwrap();
    let signing_package = SigningPackage::new(commitments_map, &message);

    let pubkey_bytes = hex::decode(pubkey_hex).unwrap();
    let pubkey = PublicKeyPackage::deserialize(&pubkey_bytes).unwrap();
    let tweaked_pubkey = pubkey.clone().tweak(None::<&[u8]>);
    let group_signature: Signature = frost_secp256k1_tr::aggregate_with_tweak(&signing_package, &sig_map, &pubkey,None).unwrap();


    // println!("[Rust] Public verifying key: {}", hex::encode(pubkey.verifying_key().serialize().unwrap()));
    // println!("[Rust] Public verifying key (Tweaked): {}", hex::encode(tweaked_pubkey.verifying_key().serialize().unwrap()));
    // println!("[Rust] Message: {}, the originally look like{:?}", hex::encode(&message),&message);
// 
    let is_signature_valid = tweaked_pubkey
    .verifying_key()
    .verify(&message, &group_signature)
    .is_ok();
    println!("[Rust] Signature valid: {}", is_signature_valid);

    Ok(hex::encode(group_signature.serialize().unwrap()))
}



 

// --- PyO3 Module Definition ---
#[pymodule]
fn rust_tss(_py: Python<'_>, m: &PyModule) -> PyResult<()> {

    // DKG related functions
    m.add_function(wrap_pyfunction!(dkg_round1, m)?)?;
    m.add_function(wrap_pyfunction!(dkg_round2, m)?)?;
    m.add_function(wrap_pyfunction!(dkg_round3, m)?)?;
    m.add_function(wrap_pyfunction!(init, m)?)?; 

    // TSS related functions
    m.add_function(wrap_pyfunction!(sign_round1, m)?)?; // round1 sign
    m.add_function(wrap_pyfunction!(sign_round2, m)?)?; // round2 sign
    m.add_function(wrap_pyfunction!(aggregate_signature, m)?)?; // Aggregate signature


    // Bitcoin related functions
    m.add_function(wrap_pyfunction!(bitcoin_related::derive_taproot_address, m)?)?; // Return taproot address
    m.add_function(wrap_pyfunction!(bitcoin_related::prepare_unsigned_tx_and_sighash, m)?)?; // Prepare unsigned tx and sighash
    m.add_function(wrap_pyfunction!(bitcoin_related::finalize_signed_tx_from_hex, m)?)?; // Finalize signed tx from hex
    Ok(())
}
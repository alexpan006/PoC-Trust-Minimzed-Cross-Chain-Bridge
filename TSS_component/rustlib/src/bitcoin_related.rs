use bitcoin::{
    blockdata::witness::Witness,
    consensus::encode::{serialize, deserialize},
    hashes::Hash,
    secp256k1::{Secp256k1, Scalar, XOnlyPublicKey},
    sighash::{Prevouts, SighashCache, TapSighashType},
    taproot,
    transaction::Version,
    Address, Amount, Network, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
};
use bitcoin::address::KnownHrp;
use bitcoin::secp256k1::schnorr::Signature as SchnorrSig;
use pyo3::prelude::*;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BtcError {
    #[error("Bitcoin hex decoding error: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("Bitcoin txid parse error: {0}")]
    TxidParse(#[from] bitcoin::hex::HexToArrayError),
    #[error("Address parse error: {0}")]
    AddressParse(#[from] bitcoin::address::ParseError),
    #[error("Secp256k1 error: {0}")]
    Secp256k1(#[from] bitcoin::secp256k1::Error),
    #[error("Taproot error: {0}")]
    Taproot(String),
    #[error("Sighash error: {0}")]
    Sighash(String),
    #[error("Invalid network string: {0}")]
    InvalidNetwork(String),
    #[error("Invalid sighash type: {0}")]
    SigHashType(String),
    #[error("Invalid signature length: expected 64 or 65 bytes, got {0}")]
    SigLength(usize),
    #[error("General error: {0}")]
    General(String),
}

impl From<BtcError> for PyErr {
    fn from(e: BtcError) -> PyErr {
        pyo3::exceptions::PyValueError::new_err(format!("Bitcoin error: {e}"))
    }
}

const DUST_LIMIT: u64 = 546;

// ===================== PyO3 Exposed Functions =====================

/// Prepare an unsigned Taproot transaction and compute the sighash.
/// Returns (unsigned_tx_hex, sighash_hex).
#[pyfunction]
pub fn prepare_unsigned_tx_and_sighash(
    utxo_txid: &str,
    utxo_vout: u32,
    prev_value: u64,
    prev_spk_hex: &str,
    to_address: &str,
    send_value: u64,
    fee_rate_sat_per_vbyte: u64,
    change_address: &str,
    network_str: &str,
) -> PyResult<(String, String)> {
    let network = parse_network(network_str)?;
    let tx = create_unsigned_tx(
        utxo_txid,
        utxo_vout,
        prev_value,
        to_address,
        send_value,
        fee_rate_sat_per_vbyte,
        change_address,
        network,
    )?;
    let prev_spk_bytes = hex::decode(prev_spk_hex)
        .map_err(|e| BtcError::General(format!("bad prev_spk_hex: {e}")))?;
    let prev_spk = ScriptBuf::from_bytes(prev_spk_bytes);

    let sighash = compute_taproot_sighash(&tx, 0, prev_value, &prev_spk)?;
    Ok((hex::encode(serialize(&tx)), hex::encode(sighash)))
}

/// Finalize a Taproot transaction: insert the signature into witness.
/// Returns the final signed transaction, hex-encoded.
#[pyfunction]
pub fn finalize_signed_tx_from_hex(
    tx_hex: &str,
    sig_hex: &str,
) -> PyResult<String> {
    let tx_bytes = hex::decode(tx_hex)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid tx hex: {e}")))?;
    let tx: Transaction = deserialize(&tx_bytes)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Failed to parse transaction: {e}")))?;
    let signed_tx_bytes = finalize_signed_tx(tx, 0, sig_hex)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Failed to finalize tx: {e}")))?;
    Ok(hex::encode(signed_tx_bytes))
}

/// Derive a Taproot address from an x-only pubkey and network.
#[pyfunction]
pub fn derive_taproot_address(x_only_hex: &str, network_str: &str) -> PyResult<String> {
    let hrp = match network_str {
        "mainnet" => KnownHrp::Mainnet,
        "testnet" => KnownHrp::Testnets,
        _ => return Err(BtcError::InvalidNetwork(format!("Invalid network: {}", network_str)).into()),
    };
    let pubkey_bytes = hex::decode(x_only_hex)
        .map_err(|e| BtcError::General(format!("Failed to decode public key: {}", e)))?;
    let pubkey_xonly = if pubkey_bytes.len() == 33 && (pubkey_bytes[0] == 0x02 || pubkey_bytes[0] == 0x03) {
        XOnlyPublicKey::from_slice(&pubkey_bytes[1..])
    } else if pubkey_bytes.len() == 32 {
        XOnlyPublicKey::from_slice(&pubkey_bytes)
    } else {
        return Err(BtcError::General(format!("Invalid public key length: {}", pubkey_bytes.len())) .into());
    }
    .map_err(|e| BtcError::SigHashType(format!("Invalid x-only public key: {}", e)))?;

    let secp = Secp256k1::new();
    let _tweaked_key = pubkey_xonly.add_tweak(&secp, &Scalar::ZERO)
        .map_err(|e| BtcError::General(format!("Failed to tweak key: {e}")))?;

    let address = Address::p2tr(&secp, pubkey_xonly, None, hrp);
    Ok(address.to_string())
}

// ===================== Internal Helpers =====================

fn parse_network(network_str: &str) -> Result<Network, BtcError> {
    match network_str {
        "mainnet" => Ok(Network::Bitcoin),
        "testnet" => Ok(Network::Testnet),
        "signet" => Ok(Network::Signet),
        "regtest" => Ok(Network::Regtest),
        _ => Err(BtcError::InvalidNetwork(network_str.to_string())),
    }
}

pub fn calculate_change(
    utxo_value: u64,
    send_value: u64,
    fee_rate_sat_per_vbyte: u64,
    with_change: bool,
) -> Result<Option<u64>, BtcError> {
    let estimated_vbytes = if with_change {
        58 + 43 + 43 + 10 // input + to + change + overhead
    } else {
        58 + 43 + 10 // input + to only + overhead
    };
    let fee = estimated_vbytes as u64 * fee_rate_sat_per_vbyte;

    if utxo_value < send_value + fee {
        return Err(BtcError::General(format!(
            "Insufficient UTXO value: {} < {} + {}",
            utxo_value, send_value, fee
        )));
    }

    let change = utxo_value - send_value - fee;
    if change < DUST_LIMIT {
        Ok(None)
    } else {
        Ok(Some(change))
    }
}

fn create_unsigned_tx(
    utxo_txid: &str,
    utxo_vout: u32,
    utxo_value: u64,
    to_address: &str,
    send_value: u64,
    fee_rate_sat_per_vbyte: u64,
    change_address: &str,
    network: Network,
) -> Result<Transaction, BtcError> {
    let txid = Txid::from_str(utxo_txid)?;
    let to_addr = Address::from_str(to_address)?.require_network(network)?;
    let to_script = to_addr.script_pubkey();

    let maybe_change_value = calculate_change(utxo_value, send_value, fee_rate_sat_per_vbyte, true)?;

    let mut tx_outs = vec![TxOut {
        value: Amount::from_sat(send_value),
        script_pubkey: to_script,
    }];

    if let Some(change_val) = maybe_change_value {
        let change_addr = Address::from_str(change_address)?.require_network(network)?;
        tx_outs.push(TxOut {
            value: Amount::from_sat(change_val),
            script_pubkey: change_addr.script_pubkey(),
        });
    }

    Ok(Transaction {
        version: Version::TWO,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: bitcoin::OutPoint { txid, vout: utxo_vout },
            script_sig: ScriptBuf::default(),
            sequence: Sequence::MAX,
            witness: Witness::default(),
        }],
        output: tx_outs,
    })
}

fn compute_taproot_sighash(
    tx: &Transaction,
    input_index: usize,
    prev_value: u64,
    prev_script_pubkey: &ScriptBuf,
) -> Result<[u8; 32], BtcError> {
    let prevout = bitcoin::TxOut {
        value: Amount::from_sat(prev_value),
        script_pubkey: prev_script_pubkey.clone(),
    };

    let mut cache = SighashCache::new(tx);
    let sighash = cache
        .taproot_key_spend_signature_hash(
            input_index,
            &Prevouts::All(&[prevout]),
            TapSighashType::All,
        )
        .map_err(|e| BtcError::Sighash(format!("Failed to compute sighash: {e}")))?;
    Ok(sighash.to_raw_hash().to_byte_array())
}

fn finalize_signed_tx(
    mut tx: Transaction,
    input_index: usize,
    schnorr_sig_hex: &str,
) -> Result<Vec<u8>, BtcError> {
    let sig_bytes = hex::decode(schnorr_sig_hex)?;
    let schnorr_sig = SchnorrSig::from_slice(&sig_bytes)?;
    let tap_sig = taproot::Signature {
        signature: schnorr_sig,
        sighash_type: TapSighashType::All,
    };
    tx.input[input_index].witness = Witness::p2tr_key_spend(&tap_sig);

    // Optionally: parse_transaction(&tx, Network::Testnet);
    Ok(serialize(&tx))
}

// ===================== Optional: Transaction Analysis Helper =====================

/// Print a human-readable summary of a transaction (for debugging).
pub fn parse_transaction(tx: &Transaction, network: Network) {
    println!("--- Transaction Analysis ---");
    println!("Txid: {}", tx.compute_txid());
    println!("Version: {:?}", tx.version);
    println!("Locktime: {:?}", tx.lock_time);
    println!("Inputs count: {}", tx.input.len());
    println!("Outputs count: {}", tx.output.len());
    println!();

    for (i, input) in tx.input.iter().enumerate() {
        println!("Input #{}", i);
        println!("  Previous txid: {}", input.previous_output.txid);
        println!("  Vout: {}", input.previous_output.vout);
        println!("  Sequence: {:?}", input.sequence);
        println!("  ScriptSig: {}", hex::encode(&input.script_sig));
        if !input.witness.is_empty() {
            println!("  Witness ({} items):", input.witness.len());
            for (w, wit_item) in input.witness.iter().enumerate() {
                println!("    [{}]: {} bytes ({})", w, wit_item.len(), hex::encode(wit_item));
            }
        } else {
            println!("  No witness");
        }
        println!();
    }

    for (i, output) in tx.output.iter().enumerate() {
        println!("Output #{}", i);
        println!("  Value: {} sats", output.value.to_sat());
        println!("  ScriptPubKey: {}", hex::encode(&output.script_pubkey));
        match Address::from_script(&output.script_pubkey, network) {
            Ok(addr) => println!("  Address: {}", addr),
            Err(_) => println!("  Address: <invalid>"),
        }
    }
    // Optionally: println!("Serialized Transaction Hex: {:?}", hex::encode(serialize(tx)));
}
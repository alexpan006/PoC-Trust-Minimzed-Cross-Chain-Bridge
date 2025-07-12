//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can have an
//! EVM-Compatible proof generated which can be verified on-chain.

// Usage Example:
// cargo run --release --bin evm -- --circuit mint --system groth16 --input-json ./input.json
use clap::{Parser, ValueEnum};
use lib_struct::{
    BitcoinTrxInfoStruct, Block, BundleInfoStruct, Chain, MerkleProof, ZkpProofFixture,
};
use sp1_sdk::{
    include_elf, HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};
use std::path::PathBuf;

/// ELF files for the Bitcoin transaction verification zkVM programs
pub const MINT_CIRCUIT_ELF: &[u8] = include_elf!("mint_circuit");
pub const BURN_CIRCUIT_ELF: &[u8] = include_elf!("burn_circuit");

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

/// Enum representing the available circuits
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum CircuitType {
    Mint,
    Burn,
}

/// The arguments for the EVM command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct EVMArgs {
    #[clap(long, value_enum, default_value = "groth16")]
    system: ProofSystem,
    #[clap(long, value_enum, default_value = "burn")]
    circuit: CircuitType,
    #[clap(long)]
    input_json: Option<PathBuf>, // Optional: path to a JSON file with public input
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = EVMArgs::parse();

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Select the circuit ELF and fixture name based on CLI argument
    let (elf, fixture_name) = match args.circuit {
        CircuitType::Mint => (MINT_CIRCUIT_ELF, "mint"),
        CircuitType::Burn => (BURN_CIRCUIT_ELF, "burn"),
    };

    // Setup the program.
    let (pk, vk) = client.setup(elf);

    // Prepare the bundle input (from file or fallback to mock data)
    let bundle_data: BundleInfoStruct = if let Some(ref path) = args.input_json {
        let file = std::fs::File::open(path).expect("failed to open input JSON");
        serde_json::from_reader(file).expect("failed to parse input JSON")
    } else {
        // Fallback to hardcoded mock data
        let mock_tx_2 = BitcoinTrxInfoStruct {
            raw_tx_hex: "010000000001015564819f67c2803761c4370d9a5fd950c8e6ff34d68ebacc47fd21413aa833ea0100000000ffffffff03e8030000000000001600141240e21b1e7877f77bfe66cc59eefb02d17a0a3f00000000000000002c6a2a3078613836456433343742384431303433353333666533306330374663343766334533623834396134329b020000000000001600144cf2f041e4acc16071306ab41414cab4c76cfd5002483045022100bf43ff7d1ae782368550cb14cc916d389277a0f103643fa352ea76ba2ccd731502205028ba84f39deb9ff71db91153c6f71e7f9f5f6df9258c29bb49ec0461785b75012103292a330133c26afde92f10737cc3e38ebcf7403b4e2232c4b65821c1aa55cdf800000000".into(),
        };
        let mock_merkle_proof = MerkleProof {
            siblings: vec![
                "cc4522617a92f7b27416f3cedad721949df7aec91d6e87f23ef2895c760e6eee".to_string(),
            ],
            pos: 1,
        };
        let block_1 = Block {
            block_hash: "00000000000002ee8b7a2baff6fc9366166d75b97301a68b0eceb3bf60f38d8f"
                .to_string(),
            version: 633618432,
            parent_hash: "0000000000000bf53edcfa982a0cbcaab1abf62660ec3ec67149df036891b32b"
                .to_string(),
            merkle_root: "214101dabc8c2b1e02999995163f31b187351c8ac1dad611e2660c2c4cae5ac6"
                .to_string(),
            timestamp: 1744638928,
            difficulty: 437256176,
            nonce: 4137494058,
        };
        let block_2 = Block {
            block_hash: "00000000000003fd04b9cb97cc0f1ce28a4588d965c595dfb4dbaf9bfd8b2a82"
                .to_string(),
            version: 770375680,
            parent_hash: "00000000000002ee8b7a2baff6fc9366166d75b97301a68b0eceb3bf60f38d8f"
                .to_string(),
            merkle_root: "b4ce4f3646fd93a8ffed7711840a09039722919c45ff1beb029d5f3027c32858"
                .to_string(),
            timestamp: 1744638928,
            difficulty: 437256176,
            nonce: 2932452395,
        };
        let block_3 = Block {
            block_hash: "0000000000000764853fd899f37e85d2765a1ec763dfd8bf2a1e739a9cad370c"
                .to_string(),
            version: 710811648,
            parent_hash: "00000000000003fd04b9cb97cc0f1ce28a4588d965c595dfb4dbaf9bfd8b2a82"
                .to_string(),
            merkle_root: "1d065531f64d5662ba174f7533bddd96632d4e530ed9df2b3d1470336f5c9daa"
                .to_string(),
            timestamp: 1744638929,
            difficulty: 437256176,
            nonce: 2559894718,
        };
        let block_4 = Block {
            block_hash: "0000000000000ef1e4b025cfb3cb6ad42482deaf8551ea2d158c23189483723a"
                .to_string(),
            version: 565084160,
            parent_hash: "0000000000000764853fd899f37e85d2765a1ec763dfd8bf2a1e739a9cad370c"
                .to_string(),
            merkle_root: "e4781238e680b8712b32696569a8f7f8a7964612cccb1cc4564c252ba0c545cf"
                .to_string(),
            timestamp: 1744638929,
            difficulty: 437256176,
            nonce: 2621199785,
        };
        let block_5 = Block {
            block_hash: "00000000000003d773169c1c0dab0a2be623b8b2357b2029d889a3078328ee5f"
                .to_string(),
            version: 565624832,
            parent_hash: "0000000000000ef1e4b025cfb3cb6ad42482deaf8551ea2d158c23189483723a"
                .to_string(),
            merkle_root: "e4b951c8dc1318c92de34759d26098c47c0b7562b05949fc741ee80b44a3d665"
                .to_string(),
            timestamp: 1744638929,
            difficulty: 437256176,
            nonce: 2556017316,
        };
        let block_6 = Block {
            block_hash: "0000000000000d76abee84857450cfec57f49c9a2bc0e5ecbf018dc72bc8bbf7"
                .to_string(),
            version: 585113600,
            parent_hash: "00000000000003d773169c1c0dab0a2be623b8b2357b2029d889a3078328ee5f"
                .to_string(),
            merkle_root: "3eae91ae2faac30f4694b548caedab64b41c2147e04e5111f0f5b43de4e39904"
                .to_string(),
            timestamp: 1744638929,
            difficulty: 437256176,
            nonce: 3028696670,
        };
        let mock_chain = Chain {
            blocks: vec![block_1, block_2, block_3, block_4, block_5, block_6],
        };
        let burner_btc_address = "tb1qzfqwyxc70pmlw7l7vmx9nmhmqtgh5z3lp3j9hf".to_string();
        BundleInfoStruct {
            merkle_proof: mock_merkle_proof,
            chains: mock_chain,
            bit_tx_info: mock_tx_2,
            burner_btc_address: burner_btc_address.into(),
        }
    };

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&bundle_data);

    println!("Proof System: {:?}", args.system);
    println!("Circuit: {:?}", args.circuit);

    // Generate the proof based on the selected proof system.
    let proof = match args.system {
        ProofSystem::Plonk => client.prove(&pk, &stdin).plonk().run(),
        ProofSystem::Groth16 => client.prove(&pk, &stdin).groth16().run(),
    }
    .expect("failed to generate proof");

    create_proof_fixture(&proof, &vk, args.system, fixture_name);
}

/// Create a fixture for the given proof.
fn create_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    system: ProofSystem,
    circuit_name: &str,
) {
    let bytes = proof.public_values.as_slice();

    let fixture = ZkpProofFixture {
        vkey: vk.bytes32().to_string(),
        public_value: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    println!("Verification Key: {}", fixture.vkey);

    // The public values are the values which are publicly committed to by the zkVM.
    println!("Public Values: {}", fixture.public_value);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture_{}.json", system, circuit_name).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}

//! Script to execute or prove Bitcoin transaction verification using SP1 zkVM for the cross-chain bridge.
//! Usage examples:
//!   RUST_LOG=info cargo run --release --bin main -- --circuit mint --execute --input-json ./input.json
//!   RUST_LOG=info cargo run --release --bin main -- --circuit burn --prove

use alloy_primitives::U256;
use alloy_sol_types::SolType;

use clap::{Parser, ValueEnum};
use lib_struct::{
    BitcoinTrxInfoStruct, Block, BundleInfoStruct, Chain, MerkleProof, ZkpMintPublicValuesStruct,
};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::path::PathBuf;

pub const MINT_CIRCUIT_ELF: &[u8] = include_elf!("mint_circuit");
pub const BURN_CIRCUIT_ELF: &[u8] = include_elf!("burn_circuit");

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum CircuitType {
    Mint,
    Burn,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,
    #[clap(long)]
    prove: bool,
    #[clap(long, value_enum, default_value = "mint")]
    circuit: CircuitType,
    #[clap(long)]
    input_json: Option<PathBuf>,
}

fn main() {
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: Specify either --execute or --prove, not both or neither");
        std::process::exit(1);
    }

    let client = ProverClient::from_env();

    // Select circuit ELF and output decoder
    let (elf, decode_output): (&[u8], fn(&[u8])) = match args.circuit {
        CircuitType::Mint => (MINT_CIRCUIT_ELF, |bytes| {
            let decoded = ZkpMintPublicValuesStruct::abi_decode(bytes).unwrap();
            println!("-------------------------------------------");
            println!("tx_id: {:?}", decoded.tx_id);
            println!("depositer eth address: {:?}", decoded.depositer_address);
            println!("amount: {:?}", decoded.amount);
            println!("is valid or not: {:?}", decoded.is_valid);
        }),
        CircuitType::Burn => (BURN_CIRCUIT_ELF, |bytes| {
            // Manual decoding to match abi_encode_zkp_burn in burn.rs

            // 1) Offset to string (skip, always 96)
            // 2) Amount (uint256)
            // 3) is_valid (uint256)
            // 4) String length (uint256)
            // 5) String bytes (padded to 32 bytes)

            if bytes.len() < 128 {
                println!("Output too short for decoding burn public values");
                return;
            }

            // Amount: bytes 32..64
            let amount = U256::from_be_slice(&bytes[32..64]);
            // is_valid: bytes 64..96
            let is_valid = U256::from_be_slice(&bytes[64..96]) == U256::from(1u8);

            // String length: bytes 96..128
            let str_len = U256::from_be_slice(&bytes[96..128]).to::<usize>();
            // String bytes: bytes 128..128+str_len
            let str_start = 128;
            let str_end = str_start + str_len;
            let btc_addr = if bytes.len() >= str_end {
                String::from_utf8_lossy(&bytes[str_start..str_end]).to_string()
            } else {
                "<decode error>".to_string()
            };

            println!("-------------------------------------------");
            println!("Burner btc address: {:?}", btc_addr);
            println!("amount: {:?}", amount);
            println!("is valid or not: {:?}", is_valid);
        }),
    };

    // Load input from JSON if provided, else fallback to mock data
    let bundle_data: BundleInfoStruct = if let Some(ref path) = args.input_json {
        let file = std::fs::File::open(path).expect("failed to open input JSON");
        serde_json::from_reader(file).expect("failed to parse input JSON")
    } else {
        // Fallback to hardcoded mock data
        let mock_tx = BitcoinTrxInfoStruct {
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
            bit_tx_info: mock_tx,
            burner_btc_address: burner_btc_address.into(),
        }
    };

    let mut stdin = SP1Stdin::new();
    stdin.write(&bundle_data);

    if args.execute {
        let (output, report) = client.execute(elf, &stdin).run().unwrap();
        decode_output(output.as_slice());
        println!("Number of cycles: {:?}", report.total_instruction_count());
        println!("Completed execution successfully!");
    } else {
        let (pk, vk) = client.setup(elf);
        let proof = client
            .prove(&pk, &stdin)
            .run()
            .expect("failed to generate proof");
        println!("Successfully generated proof!");
        decode_output(proof.public_values.as_slice());
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
    println!("Finish");
}

// For burn circuit:
// cargo run --release --bin vkey -- --circuit burn
// For mint circuit:
// cargo run --release --bin vkey -- --circuit mint

use clap::{Parser, ValueEnum};
use sp1_sdk::{include_elf, HashableKey, Prover, ProverClient};

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
    #[clap(long, value_enum, default_value = "burn")]
    circuit: CircuitType,
}

fn main() {
    let args = Args::parse();

    let elf = match args.circuit {
        CircuitType::Mint => MINT_CIRCUIT_ELF,
        CircuitType::Burn => BURN_CIRCUIT_ELF,
    };

    let prover = ProverClient::builder().cpu().build();
    let (_, vk) = prover.setup(elf);
    println!("{}", vk.bytes32());
}

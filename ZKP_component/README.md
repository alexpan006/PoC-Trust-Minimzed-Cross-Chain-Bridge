# SP1 Bitcoin-EVM zkBridge Template

This project demonstrates an end-to-end [SP1](https://github.com/succinctlabs/sp1) zero-knowledge proof system for bridging Bitcoin to EVM-compatible blockchains.  
It enables users to generate and verify zk-proofs that a Bitcoin transaction (deposit or burn) occurred and is included in a valid Bitcoin block chain, allowing secure and private minting or burning of tokens on-chain.

## Features

- **Bitcoin zkVM Circuits:**  
  - `mint`: Proves a BTC deposit to a bridge address, extracts the amount and Ethereum address from OP_RETURN, and verifies inclusion in a valid block chain.
  - `burn`: Proves a BTC burn to a burner address, extracts the amount, and verifies inclusion in a valid block chain.
- **Flexible CLI Tools:**  
  - Easily select between mint/burn circuits and proof systems (Groth16/Plonk).
  - Accepts input from JSON files or uses fallback mock data for rapid development.
  - Outputs proofs and fixtures for on-chain verification.
- **EVM Compatibility:**  
  - Generates proofs and public values that can be verified by Solidity contracts.

## Requirements

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/getting-started/install.html)

## Project Structure

- `program/`: zkVM circuits for mint and burn proofs.
- `script/`: CLI tools for proving, executing, and generating fixtures/verification keys.
- `contracts/`: Solidity contracts for on-chain verification (not detailed here).

## Usage

### 1. Build the zkVM Program

```sh
cd program
cargo prove build
```

### 2. Execute the Program (No Proof, Just Output)

```sh
cd script
cargo run --release --bin main -- --circuit mint --execute
```

- Use `--circuit burn` for the burn circuit.
- Add `--input-json ./input.json` to use custom input data.

### 3. Generate a Core Proof

```sh
cd script
cargo run --release --bin main -- --circuit mint --prove
```

- Use `--circuit burn` for the burn circuit.
- Add `--input-json ./input.json` to use custom input data.

### 4. Generate an EVM-Compatible Proof

> [!WARNING]
> You will need at least 128GB RAM to generate a Groth16 or PLONK proof.

To generate a proof suitable for on-chain verification:

```sh
cd script
cargo run --release --bin evm -- --circuit mint --system groth16 --input-json ./input.json
```

- Use `--circuit burn` for the burn circuit.
- Use `--system plonk` for a PLONK proof.
- Omitting `--input-json` will use fallback mock data.

This will generate a proof and a fixture file for Solidity verification.

### 5. Retrieve the Verification Key

To get the verification key for your on-chain contract:

```sh
cd script
cargo run --release --bin vkey -- --circuit mint
```

- Use `--circuit burn` for the burn circuit.

### 6. Example Input JSON

You can provide your own Bitcoin transaction, block chain, and proof data via a JSON file.  
See the template below:

```json
{
  "merkle_proof": {
    "siblings": ["<sibling_hash_1>", "..."],
    "pos": <position_integer>
  },
  "chains": {
    "blocks": [
      {
        "block_hash": "<block_hash_1>",
        "version": <block_version_1>,
        "parent_hash": "<parent_hash_1>",
        "merkle_root": "<merkle_root_1>",
        "timestamp": <timestamp_1>,
        "difficulty": <difficulty_1>,
        "nonce": <nonce_1>
      }
      // ...more blocks
    ]
  },
  "bit_tx_info": {
    "raw_tx_hex": "<raw_bitcoin_transaction_hex>"
  },
  "burner_btc_address": "<burner_btc_address_or_bridge_address>"
}
```

## Using the Prover Network

You can use the Succinct prover network for large or production proofs.  
For more information, see the [setup guide](https://docs.succinct.xyz/docs/generating-proofs/prover-network).

To get started, copy the example environment file:

```sh
cp .env.example .env
```

Then, set the `SP1_PROVER` environment variable to `network` and set the `NETWORK_PRIVATE_KEY`
environment variable to your whitelisted private key.

For example, to generate an EVM-compatible proof using the prover network, run:

```sh
SP1_PROVER=network NETWORK_PRIVATE_KEY=... cargo run --release --bin evm
```

---

## License

MIT

---

**For more details, see the code and comments in each script and
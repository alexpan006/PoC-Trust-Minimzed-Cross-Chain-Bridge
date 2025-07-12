# ZK-Proof Circuit for zkBTC — Contracts

This folder contains the Solidity smart contracts and local unit tests for the zkBTC bridge, based on [SP1](https://github.com/succinctlabs/sp1) zero-knowledge proof verification.  
**Note:** This repo uses the Foundry framework **only for local unit testing**. Deployment and on-chain interaction are handled in a separate Hardhat-based repository, which contains an identical `ZKBTC.sol` contract.

## Structure

- `src/ZKBTC.sol` — Main zkBTC contract, verifying SP1 zk-proofs and handling mint/burn/staking logic.
- `test/ZKBTC.t.sol` — Comprehensive unit tests for all contract logic using Foundry.
- `lib/` — External dependencies (OpenZeppelin, SP1 contracts, Forge Std).

## Requirements

- [Foundry](https://book.getfoundry.sh/getting-started/installation)

## Running Unit Tests

To run all local unit tests:

```sh
forge test -v
```

This will execute all tests in `test/ZKBTC.t.sol` and print detailed output.

## About Deployment

**Deployment and on-chain testing are not handled in this repo.**  
If you want to deploy or interact with `ZKBTC.sol` on a live network, please refer to the separate Hardhat-based repository, which contains the deployment scripts and on-chain test suite.

---

For more details, see the code and comments in each contract and test
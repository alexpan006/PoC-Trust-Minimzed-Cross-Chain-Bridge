# ZKBTC Hardhat Interaction Suite

This repository provides a collection of scripts and configuration for deploying and interacting with ZKBTC and verifier contracts using the [Hardhat](https://hardhat.org/) framework. It is designed for easy on-chain interaction, testing, and automation of workflows such as minting, burning, proof submission, and querying contract state.

---

## Features

- **Easy Deployment:**  
  Scripts for deploying verifier and ZKBTC token contracts with environment-based configuration.

- **On-chain Interaction:**  
  Scripts for minting, burning, submitting proofs, claiming rewards, and querying contract state.

- **Configurable:**  
  All sensitive and environment-specific values are managed via a `.env` file.  
  You can also modify `hardhat.config.js` to suit your own network and account settings.

- **Ready for Automation:**  
  Each script is modular and can be used in CI/CD pipelines or manual testing.

---

## Getting Started

### 1. **Install Dependencies**

```bash
npm install
```

### 2. **Configure Environment**

Copy the provided `.env` template and fill in your values:

```env
# .env template

# Verifier and Token Deployment
VERIFIER_ADDRESS=0xYourVerifierAddress
PROGRAM_VKEY_MINT=0xYourProgramVKeyMint
PROGRAM_VKEY_BURN=0xYourProgramVKeyBurn
BRIDGE_ADDRESS=your_bridge_address
STAKERS=0xStaker1,0xStaker2

# ZKBTC Contract Interaction
ZKBTC_CONTRACT_ADDRESS=0xYourZkBTCAddress
BURN_ID=0
BTC_ADDRESS=your_btc_address
BURN_AMOUNT_ZKBTC=10000000000
TARGET_ADDRESS=0xAddressToCheck

# Proof Data (for mint/burn)
PUBLIC_VALUES=0x...
PROOF_BYTES=0x...

# Hardhat Network
PRIVATE_KEY=your_private_key
RPC_SEPOLIA=https://...
RPC_ZKSYNC_SEPOLIA=https://...
```

### 3. **Modify Hardhat Configuration (Optional)**

Edit [`hardhat.config.js`](hardhat.config.js) to add or change network settings as needed.

---

## Usage

All scripts are located in the [`scripts/`](scripts) folder.  
Run any script with:

```bash
npx hardhat run scripts/<script_name>.js --network <network>
```

**Examples:**

- Deploy verifier:  
  ```bash
  npx hardhat run scripts/deploy_verifier.js --network sepolia
  ```

- Deploy ZKBTC token:  
  ```bash
  npx hardhat run scripts/deploy_token_contract.js --network zksync
  ```

- Submit proof to mint:  
  ```bash
  npx hardhat run scripts/submit_proof_to_token_mint.js --network zksync
  ```

- Initiate a burn:  
  ```bash
  npx hardhat run scripts/initiate_burn.js --network zksync
  ```

- Submit proof to burn:  
  ```bash
  npx hardhat run scripts/submit_proof_to_token_burn.js --network zksync
  ```

- Query burn request:  
  ```bash
  npx hardhat run scripts/query_burn_request.js --network zksync
  ```

- Check zkBTC balance:  
  ```bash
  npx hardhat run scripts/check_balance_token.js --network zksync
  ```

---

## Scripts Overview

- **Deployment:**  
  `deploy_verifier.js`, `deploy_token_contract.js`

- **Verifier Interaction:**  
  `interaction_test_with_verifier.js`, `submit_proof_to_verifier.js`

- **Mint/Burn Workflow:**  
  `submit_proof_to_token_mint.js`, `submit_proof_to_token_burn.js`, `initiate_burn.js`, `reclaim_burn.js`

- **Rewards:**  
  `staker_claim_reward.js`

- **Query/Utility:**  
  `query_burn_request.js`, `check_balance_token.js`, `check_vkeys.js`

---

## Notes

- All scripts use environment variables for sensitive data and configuration.
- You can freely modify `hardhat.config.js` to add new networks or change RPC endpoints.
- For zkBTC and proof-related scripts, ensure your calldata is properly ABI-encoded.

---

## License

MIT

---

**Happy hacking!**
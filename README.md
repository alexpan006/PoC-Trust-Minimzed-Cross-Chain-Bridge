# Trust-Minimized Bitcoin to Ethereum L2 Bridge (Master's Thesis PoC)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This repository contains the source code and documentation for the Proof-of-Concept (PoC) of a novel trust-minimized cross-chain bridge, developed as part of a Master's Thesis. The project demonstrates a secure and efficient architecture for bridging Bitcoin (BTC) from its Layer 1 network to an Ethereum Layer 2 rollup.

---

## 📖 Abstract

The inherent scalability limitations of the Bitcoin network pose a significant barrier to its broader utility, while Ethereum's advanced Layer 2 (L2) ecosystem offers a high-throughput, programmable environment. Cross-chain bridges designed to connect these two networks are essential but have historically introduced significant security risks and centralized trust assumptions. This project addresses the urgent need for a more secure and efficient interoperability solution by developing a novel, trust-minimized bridge architecture. The protocol employs a hybrid model, integrating a Threshold Signature Scheme (TSS) for decentralized Bitcoin custody and Zero-Knowledge Proofs (ZKPs) for verifiable inclusion of Bitcoin transactions.

For a complete and in-depth explanation of the protocol design, security analysis, and evaluation, please refer to the full thesis document located in the `/Thesis` directory.

---

## 🏛️ System Architecture Overview

This Proof-of-Concept is built from three core, independent components that work together to facilitate the cross-chain bridging process.

1.  **Decentralized Custody (TSS):** The `TSS_component` manages the decentralized custody of Bitcoin funds locked in the bridge. A council of stakers uses a Threshold Signature Scheme to collectively control the Bitcoin wallet without any single party ever holding the complete private key.

2.  **State Verification (ZKP):** The `ZKP_component` is responsible for generating cryptographic proofs. When a user deposits BTC, this component is used to create a Zero-Knowledge Proof that verifiably attests to the inclusion and validity of that transaction on the Bitcoin blockchain.

3.  **On-Chain Logic (Smart Contract):** The `Smart_Contract_component` contains the `ZKBTC.sol` token contract deployed on an Ethereum L2. This contract verifies the ZK-proofs submitted by users or operators and executes the minting (issuing `zkBTC`) or burning (redeeming `zkBTC`) of the wrapped token.

---

## 📁 Repository Structure

The repository is organized into the following main directories:

* **`/Thesis`**: Contains the full PDF document of the Master's Thesis, which provides a comprehensive background, literature review, protocol design, evaluation, and conclusion. This is the best place to start for a full understanding of the project.

* **`/Smart_Contract_component`**: Contains the `ZKBTC.sol` token smart contract, along with deployment and interaction scripts. This component is developed using the **Hardhat** framework. For detailed setup and usage instructions, please see the `README.md` file inside this directory.

* **`/ZKP_component`**: Contains the circuits for generating Zero-Knowledge Proofs for both mint and burn operations. This component is developed using the **SP1 zkVM** template. It also includes a full suite of unit tests for the smart contract written using the **Foundry** framework. For detailed instructions on generating proofs and running tests, please refer to the `README.md` inside this directory.

* **`/TSS_component`**: Contains a local simulation environment for the Threshold Signature Scheme. This component uses **Docker** to orchestrate multiple signer nodes and demonstrate the DKG (Distributed Key Generation) and signing functionality. Please see the dedicated `README.md` within this directory for setup and simulation details.

* **`README.md`**: This main README file providing a high-level overview of the entire project.

---

## 🚀 Getting Started

To explore each part of the system, please navigate to the respective component directory and follow the instructions in its dedicated `README.md` file. Each component is self-contained with its own dependencies and setup guide.

1.  **To understand the theory and design:** Read the PDF in `/Thesis`.
2.  **To explore the smart contract:** Go to `/Smart_Contract_component`.
3.  **To work with the ZK circuits and tests:** Go to `/ZKP_component`.
4.  **To simulate the TSS custody:** Go to `/TSS_component`.

---

## 📜 Citing This Work

If you use this project or the concepts from the thesis in your research, please cite the work accordingly. You can use the following BibTeX entry as a template:

```bibtex
@mastersthesis{Pan2025r,
  author  = {Kai-Yan},
  title   = {PoC of a Trust-Minimized Cross-Chain Bridge for Bitcoin Layer 1 to Ethereum Layer 2},
  school  = {University of Mannheim},
  year    = {2025},
  address = {New Taipei City, Taiwan},
  month   = {July}
}
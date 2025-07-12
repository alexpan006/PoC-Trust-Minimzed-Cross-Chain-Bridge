const hre = require("hardhat");
require('dotenv').config();

async function main() {
  // Get contract address and calldata from .env
  const contractAddress = process.env.ZKBTC_CONTRACT_ADDRESS;
  const publicValues = process.env.PUBLIC_VALUES || "0x";
  const proofBytes = process.env.PROOF_BYTES || "0x";

  if (!contractAddress) {
    throw new Error("Missing ZKBTC_CONTRACT_ADDRESS in .env");
  }

  const abi = [
    "function verifyAndMint(bytes calldata _publicValues, bytes calldata _proofBytes) external returns (bytes32, address, uint256, bool)",
    "event ProofVerifiedAndMinted(bytes32 indexed txId, address indexed depositer, uint256 amount, bool isValid)"
  ];

  const [signer] = await hre.ethers.getSigners();
  const contract = new hre.ethers.Contract(contractAddress, abi, signer);

  try {
    console.log("Submitting mint proof...");
    const tx = await contract.verifyAndMint(publicValues, proofBytes);
    const receipt = await tx.wait();

    // Parse the ProofVerifiedAndMinted event from the receipt
    let found = false;
    for (const log of receipt.logs) {
      try {
        const parsed = contract.interface.parseLog(log);
        if (parsed.name === "ProofVerifiedAndMinted") {
          found = true;
          console.log("ProofVerifiedAndMinted event:");
          console.log("txId:", parsed.args.txId);
          console.log("depositer:", parsed.args.depositer);
          console.log("amount:", parsed.args.amount.toString());
          console.log("isValid:", parsed.args.isValid);
        }
      } catch (e) {
        // Not the event we're looking for
      }
    }
    if (!found) {
      console.log("No ProofVerifiedAndMinted event found in this transaction.");
    }
  } catch (error) {
    console.error("Error occurred:", error.message);
    if (error.data) {
      try {
        const decodedError = contract.interface.parseError(error.data);
        console.log("Decoded custom error:", decodedError.name);
      } catch (decodeError) {
        console.error("Could not decode error data:", decodeError.message);
      }
    }
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
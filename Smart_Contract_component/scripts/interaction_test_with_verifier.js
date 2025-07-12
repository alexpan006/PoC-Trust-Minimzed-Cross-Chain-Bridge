const hre = require("hardhat");
require('dotenv').config();

async function main() {
  // Use contract address from .env or fallback to hardcoded value
  const contractAddress = process.env.VERIFIER_ADDRESS || "0xa27A057CAb1a4798c6242F6eE5b2416B7Cd45E5D";

  const abi = [
    "function VERSION() external pure returns (string memory)",
    "function VERIFIER_HASH() public pure returns (bytes32)"
  ];

  const [signer] = await hre.ethers.getSigners();
  console.log("Testing verifier contract at:", contractAddress);
  console.log("Using account:", signer.address);

  const contract = new hre.ethers.Contract(contractAddress, abi, signer);

  try {
    const version = await contract.VERSION();
    console.log("VERSION:", version);

    const verifierHash = await contract.VERIFIER_HASH();
    console.log("VERIFIER_HASH:", verifierHash);

    console.log("Verifier contract is deployed and responding.");
  } catch (error) {
    console.error("Error: Could not interact with verifier contract.");
    console.error(error.message || error);
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
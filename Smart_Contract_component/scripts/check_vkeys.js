const hre = require("hardhat");
require('dotenv').config();

async function main() {
  // Get contract address from .env
  const contractAddress = process.env.ZKBTC_CONTRACT_ADDRESS;
  if (!contractAddress) {
    throw new Error("Missing ZKBTC_CONTRACT_ADDRESS in .env");
  }

  const abi = [
    "function programVKey_mint() view returns (bytes32)",
    "function programVKey_burn() view returns (bytes32)"
  ];

  const [signer] = await hre.ethers.getSigners();
  const contract = new hre.ethers.Contract(contractAddress, abi, signer);

  const mintKey = await contract.programVKey_mint();
  const burnKey = await contract.programVKey_burn();

  console.log("programVKey_mint:", mintKey);
  console.log("programVKey_burn:", burnKey);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
const hre = require("hardhat");
require('dotenv').config();

async function main() {
  // Get contract and target address from .env
  const contractAddress = process.env.ZKBTC_CONTRACT_ADDRESS;
  const targetAddress = process.env.TARGET_ADDRESS;

  if (!contractAddress || !targetAddress) {
    throw new Error("Missing ZKBTC_CONTRACT_ADDRESS or TARGET_ADDRESS in .env");
  }

  const abi = [
    "function balanceOf(address) view returns (uint256)"
  ];

  const [signer] = await hre.ethers.getSigners();
  const contract = new hre.ethers.Contract(contractAddress, abi, signer);

  const balance = await contract.balanceOf(targetAddress);
  console.log(`zkBTC balance of ${targetAddress}:`, balance.toString());
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
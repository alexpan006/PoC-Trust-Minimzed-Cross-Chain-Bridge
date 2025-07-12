const hre = require("hardhat");
require('dotenv').config();

async function main() {
  // Read constructor arguments from .env
  const verifier = process.env.VERIFIER_ADDRESS;
  const programVKey_mint = process.env.PROGRAM_VKEY_MINT;
  const programVKey_burn = process.env.PROGRAM_VKEY_BURN;
  const bridge_address = process.env.BRIDGE_ADDRESS;

  // Parse stakers as a comma-separated list in .env
  const stakers = process.env.STAKERS
    ? process.env.STAKERS.split(",").map(addr => addr.trim())
    : [];

  if (!verifier || !programVKey_mint || !programVKey_burn || !bridge_address || stakers.length === 0) {
    throw new Error("Missing required environment variables. Please check your .env file.");
  }

  const zkBTC = await hre.ethers.getContractFactory("ZKBTC");
  const contract = await zkBTC.deploy(
    verifier,
    programVKey_mint,
    programVKey_burn,
    bridge_address,
    stakers
  );
  await contract.waitForDeployment();

  const address = await contract.getAddress();
  console.log("zkBTC deployed to:", address);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
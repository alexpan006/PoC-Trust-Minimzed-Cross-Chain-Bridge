const hre = require("hardhat");
require('dotenv').config();

async function main() {
  // Get contract address and BTC address from .env
  const contractAddress = process.env.ZKBTC_CONTRACT_ADDRESS;
  const btcAddress = process.env.BTC_ADDRESS;

  // Amount to burn in zkBTC units 
  const amountRequestBurnZkbtc = process.env.BURN_AMOUNT_ZKBTC
    ? BigInt(process.env.BURN_AMOUNT_ZKBTC)
    : 0; // Default: 0

  if (!contractAddress || !btcAddress) {
    throw new Error("Missing ZKBTC_CONTRACT_ADDRESS or BTC_ADDRESS in .env");
  }

  const abi = [
    "function initiateBurn(uint256 amountRequestBurnZkbtc, string calldata btcAddress) external",
    "event BurnInitiated(uint256 indexed burnId, address indexed user, uint256 amount, string btcAddress)"
  ];

  const [signer] = await hre.ethers.getSigners();
  const contract = new hre.ethers.Contract(contractAddress, abi, signer);

  // Call initiateBurn
  const tx = await contract.initiateBurn(amountRequestBurnZkbtc, btcAddress);
  const receipt = await tx.wait();

  // Parse and print the BurnInitiated event
  let found = false;
  for (const log of receipt.logs) {
    try {
      const parsed = contract.interface.parseLog(log);
      if (parsed.name === "BurnInitiated") {
        found = true;
        console.log("BurnInitiated event:");
        console.log("burnId:", parsed.args.burnId.toString());
        console.log("user:", parsed.args.user);
        console.log("amount (zkBTC units):", parsed.args.amount.toString());
        console.log("btcAddress:", parsed.args.btcAddress);
      }
    } catch (e) {
      // Not the event we're looking for
    }
  }
  if (!found) {
    console.log("No BurnInitiated event found in this transaction.");
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
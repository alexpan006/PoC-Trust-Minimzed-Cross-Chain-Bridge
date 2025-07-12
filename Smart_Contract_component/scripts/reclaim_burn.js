const hre = require("hardhat");
require('dotenv').config();

async function main() {
  // Get contract address and burnId from .env
  const contractAddress = process.env.ZKBTC_CONTRACT_ADDRESS;
  const burnId = process.env.BURN_ID ? Number(process.env.BURN_ID) : 0;

  if (!contractAddress) {
    throw new Error("Missing ZKBTC_CONTRACT_ADDRESS in .env");
  }

  const abi = [
    "function reclaimBurn(uint256 burnId) external",
    "event BurnReclaimed(uint256 indexed burnId, address indexed user, uint256 amount)"
  ];

  const [signer] = await hre.ethers.getSigners();
  const contract = new hre.ethers.Contract(contractAddress, abi, signer);

  try {
    const tx = await contract.reclaimBurn(burnId);
    const receipt = await tx.wait();

    // Parse and print the BurnReclaimed event
    let found = false;
    for (const log of receipt.logs) {
      try {
        const parsed = contract.interface.parseLog(log);
        if (parsed.name === "BurnReclaimed") {
          found = true;
          console.log("BurnReclaimed event:");
          console.log("burnId:", parsed.args.burnId.toString());
          console.log("user:", parsed.args.user);
          console.log("amount:", parsed.args.amount.toString());
        }
      } catch (e) {
        // Not the event we're looking for
      }
    }
    if (!found) {
      console.log("No BurnReclaimed event found in this transaction.");
    }
  } catch (error) {
    // Print revert reasons or custom errors
    if (error.error && error.error.data && error.error.data.message) {
      console.error("Revert reason:", error.error.data.message);
    } else {
      console.error("Error:", error.message);
    }
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
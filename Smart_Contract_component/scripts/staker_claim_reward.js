const hre = require("hardhat");
require('dotenv').config();

async function main() {
  // Get contract address from .env
  const contractAddress = process.env.ZKBTC_CONTRACT_ADDRESS;
  if (!contractAddress) {
    throw new Error("Missing ZKBTC_CONTRACT_ADDRESS in .env");
  }

  const abi = [
    "function claimStakerReward() external",
    "event StakerRewardClaimed(address indexed staker, uint256 amount)"
  ];

  const [signer] = await hre.ethers.getSigners();
  const contract = new hre.ethers.Contract(contractAddress, abi, signer);

  try {
    const tx = await contract.claimStakerReward();
    const receipt = await tx.wait();

    // Parse and print the StakerRewardClaimed event
    let found = false;
    for (const log of receipt.logs) {
      try {
        const parsed = contract.interface.parseLog(log);
        if (parsed.name === "StakerRewardClaimed") {
          found = true;
          console.log("StakerRewardClaimed event:");
          console.log("staker:", parsed.args.staker);
          console.log("amount:", parsed.args.amount.toString());
        }
      } catch (e) {
        // Not the event we're looking for
      }
    }
    if (!found) {
      console.log("No StakerRewardClaimed event found in this transaction.");
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
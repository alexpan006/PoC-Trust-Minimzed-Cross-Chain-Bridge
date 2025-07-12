const hre = require("hardhat");
require('dotenv').config();

async function main() {
  // Get contract address from .env
  const contractAddress = process.env.ZKBTC_CONTRACT_ADDRESS;
  if (!contractAddress) {
    throw new Error("Missing ZKBTC_CONTRACT_ADDRESS in .env");
  }

  // Only need the getter ABI for burnRequests
  const abi = [
    "function burnRequests(uint256) view returns (address user, uint256 total_amount, uint256 zkbtcToReimburse, uint256 exactBtcUserReceive, uint256 rewardOperator, uint256 rewardStaker, uint256 dust, string btcAddress, uint256 timestamp, bool fulfilled, bool reclaimed)"
  ];

  const [signer] = await hre.ethers.getSigners();
  const contract = new hre.ethers.Contract(contractAddress, abi, signer);

  // Get burnId from env or default to 0
  const burnId = process.env.BURN_ID ? Number(process.env.BURN_ID) : 0;
  const burnRequest = await contract.burnRequests(burnId);

  console.log(`BurnRequest #${burnId}:`);
  console.log({
    user: burnRequest.user,
    total_amount: burnRequest.total_amount.toString(),
    zkbtcToReimburse: burnRequest.zkbtcToReimburse.toString(),
    exactBtcUserReceive: burnRequest.exactBtcUserReceive.toString(),
    rewardOperator: burnRequest.rewardOperator.toString(),
    rewardStaker: burnRequest.rewardStaker.toString(),
    dust: burnRequest.dust.toString(),
    btcAddress: burnRequest.btcAddress,
    timestamp: burnRequest.timestamp.toString(),
    fulfilled: burnRequest.fulfilled,
    reclaimed: burnRequest.reclaimed
  });

  // Query current block.timestamp
  const block = await hre.ethers.provider.getBlock("latest");
  console.log("Current block.timestamp:", block.timestamp);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
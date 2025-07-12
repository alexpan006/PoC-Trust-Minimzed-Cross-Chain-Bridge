const hre = require("hardhat");
require('dotenv').config();

async function main() {
  // Get contract address from .env
  const contractAddress = process.env.ZKBTC_CONTRACT_ADDRESS;
  if (!contractAddress) {
    throw new Error("Missing ZKBTC_CONTRACT_ADDRESS in .env");
  }

  const abi = [
    "function submitBurnProof(uint256 burnId, bytes calldata _publicValues, bytes calldata _proofBytes) external",
    "event BurnFulfilled(uint256 indexed burnId, address indexed submitter)",
    "event OperatorReward(address indexed operator, uint256 amount)"
  ];

  const [signer] = await hre.ethers.getSigners();
  const contract = new hre.ethers.Contract(contractAddress, abi, signer);

  // Get burnId and proof data from .env or use defaults
  const burnId = process.env.BURN_ID ? Number(process.env.BURN_ID) : 0;
  const publicValues = process.env.PUBLIC_VALUES || "0x";
  const proofBytes = process.env.PROOF_BYTES || "0x";

  try {
    console.log("Submitting burn proof...");
    const tx = await contract.submitBurnProof(burnId, publicValues, proofBytes);
    const receipt = await tx.wait();

    // Parse and print BurnFulfilled and OperatorReward events
    for (const log of receipt.logs) {
      try {
        const parsed = contract.interface.parseLog(log);
        if (parsed.name === "BurnFulfilled") {
          console.log("BurnFulfilled event:");
          console.log("burnId:", parsed.args.burnId.toString());
          console.log("submitter:", parsed.args.submitter);
        }
        if (parsed.name === "OperatorReward") {
          console.log("OperatorReward event:");
          console.log("operator:", parsed.args.operator);
          console.log("amount:", parsed.args.amount.toString());
        }
      } catch (e) {
        // Not the event we're looking for
      }
    }
    console.log("Burn proof submitted successfully.");
  } catch (error) {
    console.error("Error occurred:", error.message);
    // Try to decode custom errors
    if (error.data) {
      try {
        const decodedError = contract.interface.parseError(error.data);
        console.log("Decoded custom error:", decodedError.name);
        switch (decodedError.name) {
          case "BurnRequestNotFound":
            console.log("Burn request does not exist.");
            break;
          case "BurnAlreadyFulfilled":
            console.log("Burn request already fulfilled.");
            break;
          case "BurnRequestExpired":
            console.log("Burn request expired.");
            break;
          case "InvalidProof":
            console.log("Proof is invalid according to public values.");
            break;
          case "OperatorSendWrongRecipent":
            console.log("BTC address in proof does not match burn request.");
            break;
          case "OperatorUnderpaid":
            console.log("Amount in proof is less than expected.");
            break;
          default:
            console.log("Other custom error:", decodedError.name);
        }
      } catch (decodeError) {
        console.error("Could not decode error data:", decodeError.message);
      }
    } else {
      console.log("No detailed error data available.");
    }
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
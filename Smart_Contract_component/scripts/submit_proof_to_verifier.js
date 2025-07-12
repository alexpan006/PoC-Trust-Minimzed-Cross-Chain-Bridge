const hre = require("hardhat");
require('dotenv').config();

async function main() {
  // Use contract address and proof data from .env or fallback to hardcoded values
  const contractAddress = process.env.VERIFIER_ADDRESS || "0x62BB284600Bc26416aB0dB1f4bfc2b5Bdd073e43";
  const programVKey = process.env.PROGRAM_VKEY || "0x003b67b489290851527f4185e0f5bd3164c70ec7cd6e59bda3eca43e9d1d9773";
  const publicValues = process.env.PUBLIC_VALUES || "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003e80000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002a746231717a6671777978633730706d6c77376c37766d78396e6d686d71746768357a336c70336a39686600000000000000000000000000000000000000000000";
  const proofBytes = process.env.PROOF_BYTES || "0x11b6a09d1630f0f6f02285326924421759151dec2e85d598f77aa4aa2f09ff72dc0695810767fc5cdd99da65d7cf7f39302bd30c370004d537e2448d04627a9cb458b6ce11f64d8b70cd3a4f59464f5a2714b0a6b425cea4e9232bc509bf884da51fe09d0a757f8c48365428afefd1e1fef89fbc6177e88a1c4fc7d13b0ae4a5c339d8030160c93edc073f7e4a1345e74337b5548a152f31cf9ca8a23691f6c917b43cef27eafd4b2e3a0e8e5eb4bacf70f273110b25afe11b2125a328084d4a73cd984b23aa328dc722d0286e6c77f66a1b8f8d19eca060a9630852d7b34d7815da128206460ade5fdc3fea5fda2174d630223f5ee8337f54da3e34107d85d4fbfc8195";

  const abi = [
    "function VERSION() external pure returns (string memory)",
    "function VERIFIER_HASH() public pure returns (bytes32)",
    "function verifyProof(bytes32 programVKey, bytes publicValues, bytes proofBytes) external view"
  ];

  const [signer] = await hre.ethers.getSigners();
  const contract = new hre.ethers.Contract(contractAddress, abi, signer);

  console.log("Testing verifier contract at:", contractAddress);
  try {
    // Optionally print version/hash for sanity check
    const version = await contract.VERSION();
    const verifierHash = await contract.VERIFIER_HASH();
    console.log("VERSION:", version);
    console.log("VERIFIER_HASH:", verifierHash);

    // Call verifyProof (will revert if data is not valid)
    await contract.verifyProof(programVKey, publicValues, proofBytes);
    console.log("✅ verifyProof call succeeded (proof accepted)");
  } catch (error) {
    console.error("❌ verifyProof call failed:");
    if (error.error && error.error.data && error.error.data.message) {
      console.error("Revert reason:", error.error.data.message);
    } else {
      console.error(error.message || error);
    }
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
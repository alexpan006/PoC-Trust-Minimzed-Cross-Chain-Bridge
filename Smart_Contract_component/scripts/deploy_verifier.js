const { ethers } = require("hardhat");

async function main() {
  const SP1Verifier = await ethers.getContractFactory("SP1Verifier");
  const verifier = await SP1Verifier.deploy(); // pass constructor args if needed
  await verifier.waitForDeployment();

  console.log("SP1 Verifier deployed to:", verifier.getAddress());
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

require("@nomicfoundation/hardhat-toolbox");
require('dotenv').config();
/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: "0.8.28",
  networks: {
    zksync: {
      url: process.env.RPC_ZKSYNC_SEPOLIA,
      accounts: [process.env.PRIVATE_KEY],
    },
    sepolia:{
      url: process.env.RPC_SEPOLIA,
      accounts: [process.env.PRIVATE_KEY],
    }
    // Add other chains as needed
  }
};

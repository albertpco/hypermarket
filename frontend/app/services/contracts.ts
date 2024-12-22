import { ethers } from "ethers";

// Contract ABIs
export const MARKET_FACTORY_ABI = [
  // Market Factory Functions
  "function createMarket(string question, uint64 expiryTimestamp, string oracleId, string collateralToken) returns (string)",
  "function getMarket(string marketId) view returns (tuple(string question, uint64 expiryTimestamp, string oracleId, string collateralToken, uint8 status, string yesTokenAddress, string noTokenAddress, bool resolvedOutcome))",
  "function listMarkets() view returns (tuple(string, tuple(string question, uint64 expiryTimestamp, string oracleId, string collateralToken, uint8 status, string yesTokenAddress, string noTokenAddress, bool resolvedOutcome))[])",
];

export const MARKET_ABI = [
  // Market Functions
  "function mintTokens(uint64 amount) returns (bool)",
  "function burnTokens(uint64 yesAmount, uint64 noAmount) returns (bool)",
  "function resolve(bool outcome) returns (bool)",
  "function claimWinnings() returns (uint64)",
  // View Functions
  "function getPosition(address account) view returns (tuple(uint64 yesTokens, uint64 noTokens, uint64 claimableWinnings))",
  "function getMarketInfo() view returns (tuple(string question, uint64 expiryTimestamp, string oracleId, string collateralToken, uint8 status, bool resolvedOutcome))",
  // Events
  "event TokensMinted(address indexed account, uint64 amount)",
  "event TokensBurned(address indexed account, uint64 yesAmount, uint64 noAmount)",
  "event MarketResolved(bool outcome)",
  "event WinningsClaimed(address indexed account, uint64 amount)",
];

// Contract Addresses (these would be the actual deployed addresses)
export const CONTRACT_ADDRESSES = {
  MARKET_FACTORY: "0x...", // Replace with actual address
  ORACLE_MANAGER: "0x...", // Replace with actual address
};

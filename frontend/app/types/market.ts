export enum MarketStatus {
  Active = "Active",
  Expired = "Expired",
  Resolved = "Resolved",
}

export interface Market {
  id: string;
  question: string;
  description: string;
  status: MarketStatus;
  expiryTimestamp: number;
  oracleId: string;
  collateralToken: string;
  volume24h: number;
  totalLiquidity: number;
  resolvedOutcome?: boolean;
}

export interface MarketPosition {
  yesTokens: number;
  noTokens: number;
  claimableWinnings: number;
}

export interface Order {
  id: string;
  marketId: string;
  trader: string;
  type: "limit" | "stop";
  side: "yes" | "no";
  amount: number;
  price: number;
  timestamp: number;
  status: "open" | "filled" | "cancelled";
}

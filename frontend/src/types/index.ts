export type Market = {
  id: string;
  question: string;
  expiry_timestamp: number;
  oracle_id: string;
  collateral_token: string;
  status: "active" | "expired" | "resolved";
  yes_token_address: string;
  no_token_address: string;
  resolved_outcome: boolean | null;
  created_at: string;
  updated_at: string;
  total_volume: string;
  current_price: string;
};

export type OrderType = "limit" | "market";
export type Side = "buy" | "sell";

export type Order = {
  id: number;
  market_id: string;
  user_address: string;
  side: Side;
  price: number;
  amount: number;
  filled_amount: number;
  status: "open" | "filled" | "cancelled";
  created_at: string;
  updated_at: string;
  tx_hash?: string;
};

export type Trade = {
  id: number;
  market_id: string;
  maker_address: string;
  taker_address: string;
  side: Side;
  price: number;
  amount: number;
  created_at: string;
  tx_hash: string;
};

export type UserPosition = {
  id: number;
  user_address: string;
  market_id: string;
  yes_tokens: string;
  no_tokens: string;
  collateral_locked: string;
  average_entry_price: string;
  unrealized_pnl: string;
  created_at: string;
  updated_at: string;
};

export type UserStats = {
  user_address: string;
  total_trades: number;
  total_volume: string;
  total_pnl: string;
  markets_participated: number;
  created_at: string;
  updated_at: string;
};

export type OrderBookLevel = {
  price: number;
  size: number;
};

export type OrderBook = {
  bids: OrderBookLevel[];
  asks: OrderBookLevel[];
};

export type PlaceOrderParams = {
  side: Side;
  price: number;
  amount: number;
  orderType: OrderType;
};

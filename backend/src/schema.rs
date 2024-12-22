use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "markets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub question: String,
    pub expiry_timestamp: i64,
    pub oracle_id: String,
    pub collateral_token: String,
    pub status: String,
    pub yes_token_address: String,
    pub no_token_address: String,
    pub resolved_outcome: Option<bool>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub total_volume: Decimal,
    pub current_price: Decimal,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_positions")]
pub struct UserPosition {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_address: String,
    pub market_id: String,
    pub yes_tokens: Decimal,
    pub no_tokens: Decimal,
    pub collateral_locked: Decimal,
    pub average_entry_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "orders")]
pub struct Order {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub market_id: String,
    pub user_address: String,
    pub side: String,
    pub price: Decimal,
    pub amount: Decimal,
    pub filled_amount: Decimal,
    pub status: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub tx_hash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "trades")]
pub struct Trade {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub market_id: String,
    pub maker_address: String,
    pub taker_address: String,
    pub side: String,
    pub price: Decimal,
    pub amount: Decimal,
    pub created_at: DateTimeWithTimeZone,
    pub tx_hash: String,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_stats")]
pub struct UserStats {
    #[sea_orm(primary_key)]
    pub user_address: String,
    pub total_trades: i32,
    pub total_volume: Decimal,
    pub total_pnl: Decimal,
    pub markets_participated: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
} 
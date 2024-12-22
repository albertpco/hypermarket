use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "markets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub question: String,
    pub expiry_timestamp: i64,
    pub oracle_id: String,
    pub collateral_token: String,
    pub status: MarketStatus,
    pub yes_token_address: String,
    pub no_token_address: String,
    pub resolved_outcome: Option<bool>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub total_volume: Decimal,
    pub current_price: Decimal,
    pub creator_address: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
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
    #[sea_orm(relation = "ManyToOne", from = "Column::MarketId", to = "super::markets::Column::Id")]
    pub market: RelationDef,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum UserPositionRelation {
    #[sea_orm(belongs_to = "super::markets::Entity", from = "Column::MarketId", to = "super::markets::Column::Id")]
    Market,
}

impl ActiveModelBehavior for UserPosition {}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "orders")]
pub struct Order {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub market_id: String,
    pub user_address: String,
    pub side: OrderSide,
    pub price: Decimal,
    pub amount: Decimal,
    pub filled_amount: Decimal,
    pub status: OrderStatus,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub tx_hash: Option<String>,
    #[sea_orm(relation = "ManyToOne", from = "Column::MarketId", to = "super::markets::Column::Id")]
    pub market: RelationDef,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum OrderRelation {
    #[sea_orm(belongs_to = "super::markets::Entity", from = "Column::MarketId", to = "super::markets::Column::Id")]
    Market,
}

impl ActiveModelBehavior for Order {}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "trades")]
pub struct Trade {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub market_id: String,
    pub maker_address: String,
    pub taker_address: String,
    pub side: OrderSide,
    pub price: Decimal,
    pub amount: Decimal,
    pub created_at: DateTimeWithTimeZone,
    pub tx_hash: String,
    #[sea_orm(relation = "ManyToOne", from = "Column::MarketId", to = "super::markets::Column::Id")]
    pub market: RelationDef,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum TradeRelation {
    #[sea_orm(belongs_to = "super::markets::Entity", from = "Column::MarketId", to = "super::markets::Column::Id")]
    Market,
}

impl ActiveModelBehavior for Trade {}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
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

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum UserStatsRelation {}

impl ActiveModelBehavior for UserStats {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum MarketStatus {
    #[sea_orm(string_value = "Active")]
    Active,
    #[sea_orm(string_value = "Expired")]
    Expired,
    #[sea_orm(string_value = "Resolved")]
    Resolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum OrderSide {
    #[sea_orm(string_value = "Buy")]
    Buy,
    #[sea_orm(string_value = "Sell")]
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum OrderStatus {
    #[sea_orm(string_value = "Open")]
    Open,
    #[sea_orm(string_value = "Filled")]
    Filled,
    #[sea_orm(string_value = "Cancelled")]
    Cancelled,
} 
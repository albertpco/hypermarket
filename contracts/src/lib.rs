use serde::{Deserialize, Serialize};

// Market types and structures
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Market {
    pub question: String,
    pub expiry_timestamp: u64,
    pub oracle_id: String,
    pub collateral_token: String,
    pub status: MarketStatus,
    pub yes_token_address: String,
    pub no_token_address: String,
    pub resolved_outcome: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MarketStatus {
    Active,
    Expired,
    Resolved,
}

// Market Factory contract interface
pub trait MarketFactory {
    fn create_market(
        &mut self,
        question: String,
        expiry_timestamp: u64,
        oracle_id: String,
        collateral_token: String,
    ) -> Result<String, String>;

    fn get_market(&self, market_id: String) -> Option<Market>;
    fn list_markets(&self) -> Vec<(String, Market)>;
}

// Market contract interface
pub trait MarketContract {
    fn mint_tokens(&mut self, amount: u64) -> Result<(), String>;
    fn burn_tokens(&mut self, yes_amount: u64, no_amount: u64) -> Result<(), String>;
    fn resolve(&mut self, outcome: bool) -> Result<(), String>;
    fn claim_winnings(&mut self) -> Result<u64, String>;
}

// Oracle Manager interface
pub trait OracleManager {
    fn register_oracle(&mut self, oracle_id: String) -> Result<(), String>;
    fn submit_outcome(&mut self, market_id: String, outcome: bool) -> Result<(), String>;
    fn get_outcome(&self, market_id: String) -> Option<bool>;
}

// Implementation modules will be added in separate files
pub mod market_factory;
pub mod market;
pub mod oracle; 
pub mod auth;
pub mod events;
pub mod hyperliquid_client;
pub mod market;
pub mod market_factory;
pub mod oracle;

pub use auth::{AuthError, AuthManager};
pub use events::{EventEmitter, EventLogger, MarketEvent, OracleEvent};
pub use hyperliquid_client::HyperliquidClient;
pub use market::{Market, MarketError, MarketStatus};
pub use market_factory::{MarketFactory, MarketFactoryError, MarketFactoryEvent, MarketFactoryState};
pub use oracle::{OracleError, OracleManager, OracleManagerState};

use async_trait::async_trait;

#[async_trait]
pub trait MarketContract {
    async fn mint_tokens(&mut self, amount: u64) -> Result<(), MarketError>;
    async fn burn_tokens(&mut self, yes_amount: u64, no_amount: u64) -> Result<(), MarketError>;
    async fn resolve(&mut self, outcome: bool) -> Result<(), MarketError>;
    async fn claim_winnings(&mut self) -> Result<u64, MarketError>;
} 
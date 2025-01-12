use crate::{
    auth::{AuthManager, AuthError},
    events::{EventEmitter, MarketEvent},
    hyperliquid_client::HyperliquidClient,
    market::{Market, MarketStatus},
};
use async_trait::async_trait;
use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarketFactoryError {
    #[error("Invalid expiry time")]
    InvalidExpiryTime,
    #[error("Invalid oracle")]
    InvalidOracle,
    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Unauthorized")]
    Unauthorized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketFactoryEvent {
    MarketCreated {
        market_id: String,
        question: String,
        expiry_timestamp: u64,
        oracle_id: Address,
        collateral_token: String,
        creator: Address,
    },
    OracleAdded {
        oracle_address: Address,
        timestamp: u64,
    },
}

#[async_trait]
pub trait MarketFactory {
    async fn create_market(
        &mut self,
        question: String,
        expiry_timestamp: u64,
        oracle_id: Address,
        collateral_token: String,
    ) -> Result<String, MarketFactoryError>;

    fn get_market(&self, market_id: String) -> Option<Market>;
    fn list_markets(&self) -> Vec<(String, Market)>;
    async fn add_oracle(&mut self, oracle_address: Address) -> Result<(), MarketFactoryError>;
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MarketFactoryState {
    markets: HashMap<String, Market>,
    oracle_whitelist: Vec<Address>,
    next_market_id: u64,
    auth_manager: Arc<AuthManager>,
    event_emitter: Arc<dyn EventEmitter>,
    listing_fee: U256,
    client: HyperliquidClient,
}

impl MarketFactoryState {
    pub async fn new(
        _api_url: &str,
        auth_manager: Arc<AuthManager>,
        event_emitter: Arc<dyn EventEmitter>,
        listing_fee: U256,
    ) -> Result<Self, MarketFactoryError> {
        let client = HyperliquidClient::new(auth_manager.clone());

        Ok(Self {
            markets: HashMap::new(),
            oracle_whitelist: Vec::new(),
            next_market_id: 0,
            auth_manager,
            event_emitter,
            listing_fee,
            client,
        })
    }

    fn generate_market_id(&mut self) -> String {
        let id = self.next_market_id;
        self.next_market_id += 1;
        format!("MARKET_{}", id)
    }

    #[allow(dead_code)]
    fn generate_token_address(market_id: &str, is_yes: bool) -> String {
        let mut hasher = Keccak256::new();
        hasher.update(market_id.as_bytes());
        hasher.update(if is_yes { b"YES\0\0" } else { b"NO\0\0\0" });
        format!("0x{}", hex::encode(hasher.finalize()))
    }

    async fn get_caller_address(&self) -> Result<Address, MarketFactoryError> {
        self.auth_manager.get_current_address().map_err(MarketFactoryError::AuthError)
    }
}

#[async_trait]
impl MarketFactory for MarketFactoryState {
    async fn create_market(
        &mut self,
        question: String,
        expiry_timestamp: u64,
        oracle_id: Address,
        collateral_token: String,
    ) -> Result<String, MarketFactoryError> {
        if expiry_timestamp <= std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
        {
            return Err(MarketFactoryError::InvalidExpiryTime);
        }

        if !self.oracle_whitelist.contains(&oracle_id) {
            return Err(MarketFactoryError::InvalidOracle);
        }

        let caller_address = self.get_caller_address().await?;

        // Generate market ID and create token markets
        let market_id = self.generate_market_id();
        let (yes_token_address, no_token_address) = self.client
            .create_market_pair(&market_id, &collateral_token)
            .await
            .map_err(|e| MarketFactoryError::ApiError(e))?;

        // Create market
        let market = Market {
            question: question.clone(),
            expiry_timestamp,
            oracle_id: format!("{:?}", oracle_id),
            collateral_token: format!("{:?}", collateral_token),
            status: MarketStatus::Active,
            yes_token_address: yes_token_address.clone(),
            no_token_address: no_token_address.clone(),
            resolved_outcome: None,
        };

        self.markets.insert(market_id.clone(), market.clone());

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::MarketCreated {
            market_id: market_id.clone(),
            creator: caller_address,
            question,
            expiry_timestamp,
            oracle_id,
            yes_token: yes_token_address,
            no_token: no_token_address,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        Ok(market_id)
    }

    fn get_market(&self, market_id: String) -> Option<Market> {
        self.markets.get(&market_id).cloned()
    }

    fn list_markets(&self) -> Vec<(String, Market)> {
        self.markets
            .iter()
            .map(|(id, market)| (id.clone(), market.clone()))
            .collect()
    }

    async fn add_oracle(&mut self, oracle_id: Address) -> Result<(), MarketFactoryError> {
        let caller_address = self.get_caller_address().await?;
        if caller_address != self.auth_manager.get_current_address()? {
            return Err(MarketFactoryError::Unauthorized);
        }

        self.oracle_whitelist.push(oracle_id);

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::OracleAdded {
            oracle_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventLogger;
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::signers::LocalWallet;

    async fn setup_test_factory() -> (MarketFactoryState, LocalWallet) {
        // Create test wallet
        let wallet: LocalWallet = SigningKey::random(&mut rand::thread_rng()).into();
        let private_key = wallet.signer().to_bytes().to_vec();
        let private_key_hex = hex::encode(private_key);

        // Create auth manager
        let auth_manager = Arc::new(
            AuthManager::new("http://localhost:8545")
                .await
                .unwrap()
        );
        auth_manager.as_ref()
            .connect_wallet(&private_key_hex)
            .await
            .unwrap();

        // Create event logger
        let event_logger = Arc::new(EventLogger::new(true, false, None));

        let factory = MarketFactoryState::new(
            "http://localhost:8080",
            auth_manager,
            event_logger,
            U256::from(100),
        )
        .await
        .unwrap();

        (factory, wallet)
    }

    #[tokio::test]
    async fn test_create_market() {
        let (mut factory, wallet) = setup_test_factory().await;
        
        // Add test oracle
        factory.add_oracle(wallet.address()).await.unwrap();

        // Test market creation
        let future_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 86400; // 1 day in future

        let result = factory
            .create_market(
                "Will ETH price be above $2000 tomorrow?".to_string(),
                future_timestamp,
                wallet.address(),
                "USDC".to_string(),
            )
            .await;

        assert!(result.is_ok());
        let market_id = result.unwrap();
        
        // Verify market was created
        let market = factory.get_market(market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Active);
    }

    #[tokio::test]
    async fn test_create_market_invalid_oracle() {
        let (mut factory, _) = setup_test_factory().await;
        
        // Test market creation with unregistered oracle
        let future_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 86400;

        let result = factory
            .create_market(
                "Will ETH price be above $2000 tomorrow?".to_string(),
                future_timestamp,
                Address::zero(),
                "USDC".to_string(),
            )
            .await;

        assert!(matches!(result, Err(MarketFactoryError::InvalidOracle)));
    }
} 
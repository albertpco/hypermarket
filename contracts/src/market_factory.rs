use crate::{auth::{AuthManager, AuthError}, Market, MarketFactory, MarketStatus};
use ethers::types::{Address, U256};
use hyperliquid_rust::{
    types::{Asset, AssetInfo, MarketMeta},
    HyperliquidApi, HyperliquidError,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use async_trait::async_trait;

#[derive(Error, Debug)]
pub enum MarketFactoryError {
    #[error("Invalid expiry time")]
    InvalidExpiryTime,
    #[error("Invalid oracle")]
    InvalidOracle,
    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),
    #[error("Hyperliquid error: {0}")]
    HyperliquidError(#[from] HyperliquidError),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MarketFactoryState {
    markets: HashMap<String, Market>,
    oracle_whitelist: Vec<Address>,
    next_market_id: u64,
    api: HyperliquidApi,
    auth_manager: Arc<AuthManager>,
}

impl MarketFactoryState {
    pub async fn new(api_url: &str, auth_manager: Arc<AuthManager>) -> Result<Self, MarketFactoryError> {
        let api = HyperliquidApi::new(api_url)
            .map_err(MarketFactoryError::HyperliquidError)?;

        Ok(Self {
            markets: HashMap::new(),
            oracle_whitelist: Vec::new(),
            next_market_id: 0,
            api,
            auth_manager,
        })
    }

    fn generate_market_id(&mut self) -> String {
        let id = self.next_market_id;
        self.next_market_id += 1;
        format!("MARKET_{}", id)
    }

    fn generate_token_address(market_id: &str, is_yes: bool) -> String {
        let mut hasher = Keccak256::new();
        hasher.update(market_id.as_bytes());
        hasher.update(if is_yes { b"YES" } else { b"NO" });
        format!("0x{}", hex::encode(hasher.finalize()))
    }

    async fn create_market_pair(
        &self,
        market_id: &str,
        collateral_token: &str,
    ) -> Result<(String, String), MarketFactoryError> {
        let yes_token_address = Self::generate_token_address(market_id, true);
        let no_token_address = Self::generate_token_address(market_id, false);

        let signed_request = self.create_signed_request("create_market").await?;

        // Create YES token market
        let yes_market = MarketMeta {
            coin: yes_token_address.clone(),
            oracle_source: "hypermarket".to_string(),
            oracle_provider: market_id.to_string(),
            base_asset: Asset {
                coin: yes_token_address.clone(),
                info: AssetInfo {
                    decimals: 18,
                    min_size: U256::from(1),
                    min_tick: U256::from(1),
                    ..Default::default()
                },
            },
            quote_asset: Asset {
                coin: collateral_token.to_string(),
                info: AssetInfo::default(),
            },
        };

        // Create NO token market
        let no_market = MarketMeta {
            coin: no_token_address.clone(),
            oracle_source: "hypermarket".to_string(),
            oracle_provider: market_id.to_string(),
            base_asset: Asset {
                coin: no_token_address.clone(),
                info: AssetInfo {
                    decimals: 18,
                    min_size: U256::from(1),
                    min_tick: U256::from(1),
                    ..Default::default()
                },
            },
            quote_asset: Asset {
                coin: collateral_token.to_string(),
                info: AssetInfo::default(),
            },
        };

        // Create markets on Hyperliquid
        self.api
            .create_market_with_auth(&signed_request, &yes_market)
            .await
            .map_err(MarketFactoryError::HyperliquidError)?;

        self.api
            .create_market_with_auth(&signed_request, &no_market)
            .await
            .map_err(MarketFactoryError::HyperliquidError)?;

        Ok((yes_token_address, no_token_address))
    }

    async fn create_signed_request(&self, action: &str) -> Result<String, MarketFactoryError> {
        let (message, signature) = self.auth_manager
            .create_signed_request(action)
            .await
            .map_err(MarketFactoryError::AuthError)?;

        let request = serde_json::json!({
            "message": message,
            "signature": signature,
        });

        Ok(serde_json::to_string(&request)
            .map_err(|e| MarketFactoryError::HyperliquidError(HyperliquidError::SerializationError(e.to_string())))?)
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
        // Validate inputs
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if expiry_timestamp <= current_time {
            return Err(MarketFactoryError::InvalidExpiryTime);
        }

        if !self.oracle_whitelist.contains(&oracle_id) {
            return Err(MarketFactoryError::InvalidOracle);
        }

        // Generate market ID and create token markets
        let market_id = self.generate_market_id();
        let (yes_token_address, no_token_address) = self.create_market_pair(&market_id, &collateral_token).await?;

        // Create market
        let market = Market {
            question,
            expiry_timestamp,
            oracle_id: format!("{:?}", oracle_id),
            collateral_token,
            status: MarketStatus::Active,
            yes_token_address,
            no_token_address,
            resolved_outcome: None,
        };

        self.markets.insert(market_id.clone(), market);

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

    async fn add_oracle(&mut self, oracle_address: Address) -> Result<(), MarketFactoryError> {
        // Verify caller has permission to add oracles
        let caller = self.get_caller_address().await?;
        let signed_request = self.create_signed_request("add_oracle").await?;

        // Verify with Hyperliquid that caller has permission
        self.api
            .verify_admin_with_auth(&signed_request, &format!("{:?}", caller))
            .await
            .map_err(MarketFactoryError::HyperliquidError)?;

        if !self.oracle_whitelist.contains(&oracle_address) {
            self.oracle_whitelist.push(oracle_address);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let factory = MarketFactoryState::new(
            "http://localhost:8080",
            auth_manager,
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
} 
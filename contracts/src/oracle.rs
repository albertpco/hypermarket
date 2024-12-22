use crate::{
    auth::{AuthManager, AuthError},
    events::{EventEmitter, OracleEvent},
    OracleManager,
};
use ethers::{
    types::{Address, Signature, H256},
    utils::keccak256,
};
use hyperliquid_rust::{
    types::{OraclePrice, OracleUpdate},
    HyperliquidApi, HyperliquidError,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use async_trait::async_trait;

#[derive(Error, Debug)]
pub enum OracleError {
    #[error("Oracle not registered")]
    OracleNotRegistered,
    #[error("Market outcome already submitted")]
    OutcomeAlreadySubmitted,
    #[error("Invalid oracle signature")]
    InvalidSignature,
    #[error("Market not found")]
    MarketNotFound,
    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),
    #[error("Hyperliquid error: {0}")]
    HyperliquidError(#[from] HyperliquidError),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OracleSubmission {
    pub oracle_id: Address,
    pub market_id: String,
    pub outcome: bool,
    pub timestamp: u64,
    pub signature: Signature,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OracleManagerState {
    oracles: Vec<Address>,
    outcomes: HashMap<String, bool>,
    auth_manager: Arc<AuthManager>,
    event_emitter: Arc<dyn EventEmitter>,
    admin: Address,
}

impl OracleManagerState {
    pub async fn new(
        auth_manager: Arc<AuthManager>,
        event_emitter: Arc<dyn EventEmitter>,
        admin: Address,
    ) -> Result<Self, OracleError> {
        Ok(Self {
            oracles: Vec::new(),
            outcomes: HashMap::new(),
            auth_manager,
            event_emitter,
            admin,
        })
    }

    async fn get_caller_address(&self) -> Result<Address, OracleError> {
        self.auth_manager.get_current_address().map_err(OracleError::AuthError)
    }
}

#[async_trait]
impl OracleManager for OracleManagerState {
    async fn register_oracle(&mut self, oracle_id: String) -> Result<(), OracleError> {
        let caller_address = self.get_caller_address().await?;
        if caller_address != self.admin {
            return Err(OracleError::AuthError(AuthError::Unauthorized));
        }

        let oracle_address = oracle_id.parse::<Address>().map_err(|_| OracleError::InvalidSignature)?;
        self.oracles.push(oracle_address);
        Ok(())
    }

    async fn submit_outcome(&mut self, market_id: String, outcome: bool) -> Result<(), OracleError> {
        let caller_address = self.get_caller_address().await?;
        if !self.oracles.contains(&caller_address) {
            return Err(OracleError::Unauthorized);
        }

        if self.outcomes.contains_key(&market_id) {
            return Err(OracleError::OutcomeAlreadySubmitted);
        }

        self.outcomes.insert(market_id.clone(), outcome);

        // Emit event
        self.event_emitter.emit_oracle_event(OracleEvent::OutcomeSubmitted {
            market_id,
            outcome,
            oracle: caller_address,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        Ok(())
    }

    fn get_outcome(&self, market_id: String) -> Option<bool> {
        self.outcomes.get(&market_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventLogger;
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::signers::LocalWallet;

    async fn setup_test_oracle() -> (OracleManagerState, LocalWallet) {
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

        let admin_address = wallet.address();

        let manager = OracleManagerState::new(
            auth_manager,
            event_logger,
            admin_address,
        )
        .await
        .unwrap();

        (manager, wallet)
    }

    #[tokio::test]
    async fn test_register_oracle() {
        let (mut manager, wallet) = setup_test_oracle().await;
        let oracle_address = wallet.address();
        
        // Register oracle
        let result = manager.register_oracle(format!("{:?}", oracle_address)).await;
        assert!(result.is_ok());
        
        // Verify oracle is registered
        assert!(manager.oracles.contains(&oracle_address));
    }

    #[tokio::test]
    async fn test_register_oracle_unauthorized() {
        let (mut manager, wallet) = setup_test_oracle().await;
        let other_wallet: LocalWallet = SigningKey::random(&mut rand::thread_rng()).into();
        let other_address = other_wallet.address();

        // Try to register oracle with unauthorized address
        let result = manager.register_oracle(format!("{:?}", other_address)).await;
        assert!(matches!(result, Err(OracleError::AuthError(AuthError::Unauthorized))));
    }

    #[tokio::test]
    async fn test_submit_outcome() {
        let (mut manager, wallet) = setup_test_oracle().await;
        let oracle_address = wallet.address();

        // Register oracle
        manager.register_oracle(format!("{:?}", oracle_address)).await.unwrap();

        // Create a signature
        let message = "test_message";
        let signature = wallet.sign_message(message).await.unwrap();
        
        // Submit outcome
        let result = manager.submit_outcome(
            "test_market".to_string(),
            true,
        ).await;
        assert!(result.is_ok());
        
        // Verify outcome
        let outcome = manager.get_outcome("test_market".to_string());
        assert_eq!(outcome, Some(true));
    }

    #[tokio::test]
    async fn test_submit_outcome_unregistered_oracle() {
        let (mut manager, wallet) = setup_test_oracle().await;
        
        // Try to submit outcome with unregistered oracle
        let result = manager.submit_outcome(
            "test_market".to_string(),
            true,
        ).await;
        assert!(matches!(result, Err(OracleError::OracleNotRegistered)));
    }

    #[tokio::test]
    async fn test_submit_outcome_already_submitted() {
        let (mut manager, wallet) = setup_test_oracle().await;
        let oracle_address = wallet.address();

        // Register oracle
        manager.register_oracle(format!("{:?}", oracle_address)).await.unwrap();
        
        // Submit outcome first time
        manager.submit_outcome(
            "test_market".to_string(),
            true,
        ).await.unwrap();

        // Try to submit outcome again
        let result = manager.submit_outcome(
            "test_market".to_string(),
            false,
        ).await;
        assert!(matches!(result, Err(OracleError::OutcomeAlreadySubmitted)));
    }

    #[tokio::test]
    async fn test_submit_outcome_unauthorized() {
        let (mut manager, _) = setup_test_oracle().await;
        let (_, wallet) = setup_test_oracle().await;
        let oracle_address = wallet.address();

        // Try to submit outcome with unauthorized oracle
        let result = manager.submit_outcome(
            "test_market".to_string(),
            true,
        ).await;
        assert!(matches!(result, Err(OracleError::Unauthorized)));
    }
} 
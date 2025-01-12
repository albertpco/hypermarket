use crate::{
    auth::{AuthManager, AuthError},
    events::{EventEmitter, OracleEvent},
};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum OracleError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid market")]
    InvalidMarket,
    #[error("Market already resolved")]
    MarketAlreadyResolved,
    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),
}

#[async_trait]
pub trait OracleManager: Send + Sync {
    async fn submit_outcome(&self, market_id: String, outcome: bool) -> Result<(), OracleError>;
    fn get_outcome(&self, market_id: String) -> Option<bool>;
}

pub struct OracleManagerState {
    outcomes: RwLock<HashMap<String, bool>>,
    auth_manager: Arc<AuthManager>,
    event_emitter: Arc<dyn EventEmitter>,
}

impl OracleManagerState {
    pub fn new(auth_manager: Arc<AuthManager>, event_emitter: Arc<dyn EventEmitter>) -> Self {
        Self {
            outcomes: RwLock::new(HashMap::new()),
            auth_manager,
            event_emitter,
        }
    }
}

#[async_trait]
impl OracleManager for OracleManagerState {
    async fn submit_outcome(&self, market_id: String, outcome: bool) -> Result<(), OracleError> {
        let mut outcomes = self.outcomes.write().await;
        
        if outcomes.contains_key(&market_id) {
            return Err(OracleError::MarketAlreadyResolved);
        }

        outcomes.insert(market_id.clone(), outcome);

        // Emit event
        let user_address = self.auth_manager.get_current_address()
            .map_err(OracleError::AuthError)?;

        self.event_emitter.emit_oracle_event(OracleEvent::OutcomeSubmitted {
            market_id: market_id.clone(),
            oracle: user_address,
            outcome,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        Ok(())
    }

    fn get_outcome(&self, market_id: String) -> Option<bool> {
        self.outcomes
            .try_read()
            .ok()?
            .get(&market_id)
            .copied()
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

        let oracle_manager = OracleManagerState::new(auth_manager, event_logger);

        (oracle_manager, wallet)
    }

    #[tokio::test]
    async fn test_submit_outcome() {
        let (oracle_manager, _) = setup_test_oracle().await;
        let market_id = "test_market".to_string();
        
        let result = oracle_manager.submit_outcome(market_id.clone(), true).await;
        assert!(result.is_ok());
        
        let outcome = oracle_manager.get_outcome(market_id);
        assert_eq!(outcome, Some(true));
    }

    #[tokio::test]
    async fn test_submit_duplicate_outcome() {
        let (oracle_manager, _) = setup_test_oracle().await;
        let market_id = "test_market".to_string();
        
        oracle_manager.submit_outcome(market_id.clone(), true).await.unwrap();
        let result = oracle_manager.submit_outcome(market_id, false).await;
        
        assert!(matches!(result, Err(OracleError::MarketAlreadyResolved)));
    }

    #[tokio::test]
    async fn test_get_nonexistent_outcome() {
        let (oracle_manager, _) = setup_test_oracle().await;
        let market_id = "nonexistent_market".to_string();
        
        let outcome = oracle_manager.get_outcome(market_id);
        assert_eq!(outcome, None);
    }
} 
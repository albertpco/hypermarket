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
pub struct OracleInfo {
    pub address: Address,
    pub public_key: H256,
    pub reputation_score: u32,
    pub total_submissions: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OracleManagerState {
    registered_oracles: HashMap<Address, OracleInfo>,
    market_outcomes: HashMap<String, OracleSubmission>,
    api: HyperliquidApi,
    auth_manager: Arc<AuthManager>,
    event_emitter: Arc<dyn EventEmitter>,
}

impl OracleManagerState {
    pub async fn new(
        api_url: &str,
        auth_manager: Arc<AuthManager>,
        event_emitter: Arc<dyn EventEmitter>,
    ) -> Result<Self, OracleError> {
        let api = HyperliquidApi::new(api_url)
            .map_err(OracleError::HyperliquidError)?;

        Ok(Self {
            registered_oracles: HashMap::new(),
            market_outcomes: HashMap::new(),
            api,
            auth_manager,
            event_emitter,
        })
    }

    fn verify_signature(&self, submission: &OracleSubmission) -> bool {
        if let Some(oracle_info) = self.registered_oracles.get(&submission.oracle_id) {
            // Create message hash
            let message = format!(
                "{}:{}:{}:{}",
                submission.market_id,
                submission.outcome,
                submission.timestamp,
                submission.oracle_id
            );
            let message_hash = keccak256(message.as_bytes());

            // Verify signature
            submission
                .signature
                .verify(message_hash, oracle_info.address)
                .is_ok()
        } else {
            false
        }
    }

    async fn submit_to_hyperliquid(
        &self,
        market_id: &str,
        outcome: bool,
        timestamp: u64,
    ) -> Result<(), OracleError> {
        let signed_request = self.create_signed_request("submit_oracle_update").await?;

        let oracle_update = OracleUpdate {
            coin: market_id.to_string(),
            timestamp,
            price: OraclePrice::Binary(outcome),
        };

        self.api
            .submit_oracle_update_with_auth(&signed_request, &oracle_update)
            .await
            .map_err(OracleError::HyperliquidError)
    }

    async fn create_signed_request(&self, action: &str) -> Result<String, OracleError> {
        let (message, signature) = self.auth_manager
            .create_signed_request(action)
            .await
            .map_err(OracleError::AuthError)?;

        let request = serde_json::json!({
            "message": message,
            "signature": signature,
        });

        Ok(serde_json::to_string(&request)
            .map_err(|e| OracleError::HyperliquidError(HyperliquidError::SerializationError(e.to_string())))?)
    }

    async fn get_caller_address(&self) -> Result<Address, OracleError> {
        self.auth_manager.get_current_address().map_err(OracleError::AuthError)
    }
}

#[async_trait]
impl OracleManager for OracleManagerState {
    async fn register_oracle(
        &mut self,
        address: Address,
        public_key: H256,
    ) -> Result<(), OracleError> {
        // Verify caller has admin permissions
        let caller = self.get_caller_address().await?;
        let signed_request = self.create_signed_request("register_oracle").await?;

        // Verify with Hyperliquid that caller has permission
        self.api
            .verify_admin_with_auth(&signed_request, &format!("{:?}", caller))
            .await
            .map_err(OracleError::HyperliquidError)?;

        if !self.registered_oracles.contains_key(&address) {
            let oracle_info = OracleInfo {
                address,
                public_key,
                reputation_score: 100, // Initial score
                total_submissions: 0,
            };
            self.registered_oracles.insert(address, oracle_info);

            // Emit event
            self.event_emitter.emit_oracle_event(OracleEvent::OracleRegistered {
                oracle_id: address,
                registrar: caller,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
        Ok(())
    }

    async fn submit_outcome(
        &mut self,
        market_id: String,
        outcome: bool,
        signature: Signature,
    ) -> Result<(), OracleError> {
        let oracle_id = self.get_caller_address().await?;
        
        // Verify oracle is registered
        if !self.registered_oracles.contains_key(&oracle_id) {
            return Err(OracleError::OracleNotRegistered);
        }

        // Check if outcome already submitted
        if self.market_outcomes.contains_key(&market_id) {
            return Err(OracleError::OutcomeAlreadySubmitted);
        }

        // Create submission with current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let submission = OracleSubmission {
            oracle_id,
            market_id: market_id.clone(),
            outcome,
            timestamp,
            signature,
        };

        // Verify signature
        if !self.verify_signature(&submission) {
            return Err(OracleError::InvalidSignature);
        }

        // Submit to Hyperliquid
        self.submit_to_hyperliquid(&market_id, outcome, timestamp).await?;

        // Store outcome
        self.market_outcomes.insert(market_id.clone(), submission);

        // Update oracle stats and emit reputation update event
        if let Some(oracle_info) = self.registered_oracles.get_mut(&oracle_id) {
            let old_score = oracle_info.reputation_score;
            oracle_info.total_submissions += 1;
            // Update reputation score based on submission (simplified)
            oracle_info.reputation_score = oracle_info.reputation_score.saturating_add(1);

            self.event_emitter.emit_oracle_event(OracleEvent::ReputationUpdated {
                oracle_id,
                old_score,
                new_score: oracle_info.reputation_score,
                timestamp,
            });
        }

        // Emit outcome submission event
        self.event_emitter.emit_oracle_event(OracleEvent::OutcomeSubmitted {
            market_id,
            oracle_id,
            outcome,
            timestamp,
        });

        Ok(())
    }

    fn get_outcome(&self, market_id: String) -> Option<bool> {
        self.market_outcomes.get(&market_id).map(|s| s.outcome)
    }

    fn get_oracle_info(&self, oracle_address: Address) -> Option<(Address, u32, u32)> {
        self.registered_oracles.get(&oracle_address).map(|info| {
            (info.address, info.reputation_score, info.total_submissions)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventLogger;
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::signers::{LocalWallet, Signer};

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

        let oracle_manager = OracleManagerState::new(
            "http://localhost:8080",
            auth_manager,
            event_logger,
        )
        .await
        .unwrap();

        (oracle_manager, wallet)
    }

    #[tokio::test]
    async fn test_oracle_registration() {
        let (mut manager, wallet) = setup_test_oracle().await;
        let public_key = H256::from_slice(&wallet.signer().verifying_key().to_bytes());

        // Register oracle
        let result = manager.register_oracle(
            wallet.address(),
            public_key,
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_outcome_submission() {
        let (mut manager, wallet) = setup_test_oracle().await;
        let public_key = H256::from_slice(&wallet.signer().verifying_key().to_bytes());
        
        // Register oracle
        manager.register_oracle(
            wallet.address(),
            public_key,
        ).await.unwrap();

        // Create and sign test message
        let message = format!(
            "test_market:true:1234567890:{}",
            wallet.address()
        );
        let signature = wallet
            .sign_message(message.as_bytes())
            .await
            .unwrap();
        
        // Submit outcome
        let result = manager.submit_outcome(
            "test_market".to_string(),
            true,
            signature,
        ).await;
        assert!(result.is_ok());
        
        // Verify outcome
        let outcome = manager.get_outcome("test_market".to_string());
        assert_eq!(outcome, Some(true));
    }
} 
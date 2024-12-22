use crate::events::{EventEmitter, AuthEvent};
use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, Signature, H256},
};
use hyperliquid_rust::HyperliquidError;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Not authenticated")]
    NotAuthenticated,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Hyperliquid error: {0}")]
    HyperliquidError(#[from] HyperliquidError),
}

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub address: Address,
    pub wallet: LocalWallet,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthMessage {
    pub timestamp: u64,
    pub action: String,
    pub nonce: u64,
}

pub type Client = Arc<SignerMiddleware<Provider<Http>, LocalWallet>>;

pub struct AuthManager {
    provider: Provider<Http>,
    current_user: Option<AuthenticatedUser>,
    event_emitter: Arc<dyn EventEmitter>,
}

impl AuthManager {
    pub async fn new(rpc_url: &str, event_emitter: Arc<dyn EventEmitter>) -> Result<Self, AuthError> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| AuthError::ProviderError(e.to_string()))?;

        Ok(Self {
            provider,
            current_user: None,
            event_emitter,
        })
    }

    pub async fn connect_wallet(&mut self, private_key: &str) -> Result<AuthenticatedUser, AuthError> {
        let wallet = private_key.parse::<LocalWallet>()
            .map_err(|e| AuthError::ProviderError(e.to_string()))?;

        let user = AuthenticatedUser {
            address: wallet.address(),
            wallet: wallet.clone(),
        };

        self.current_user = Some(user.clone());

        // Emit authentication event
        self.event_emitter.emit_auth_event(AuthEvent::UserAuthenticated {
            user: user.address,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        Ok(user)
    }

    pub fn get_current_user(&self) -> Option<&AuthenticatedUser> {
        self.current_user.as_ref()
    }

    pub fn get_current_address(&self) -> Result<Address, AuthError> {
        self.current_user
            .as_ref()
            .map(|user| user.address)
            .ok_or(AuthError::NotAuthenticated)
    }

    pub async fn sign_message(&self, message: &str) -> Result<Signature, AuthError> {
        let user = self.current_user
            .as_ref()
            .ok_or(AuthError::NotAuthenticated)?;

        let signature = user.wallet
            .sign_message(message)
            .await
            .map_err(|e| AuthError::ProviderError(e.to_string()))?;

        // Emit signature creation event
        self.event_emitter.emit_auth_event(AuthEvent::SignatureCreated {
            user: user.address,
            action: message.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        Ok(signature)
    }

    pub fn create_auth_message(&self, action: &str) -> AuthMessage {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        AuthMessage {
            timestamp,
            action: action.to_string(),
            nonce: rand::random(),
        }
    }

    pub async fn create_signed_request(&self, action: &str) -> Result<(AuthMessage, Signature), AuthError> {
        let message = self.create_auth_message(action);
        let message_str = serde_json::to_string(&message)
            .map_err(|e| AuthError::ProviderError(e.to_string()))?;
        
        let signature = self.sign_message(&message_str).await?;
        Ok((message, signature))
    }

    pub fn verify_signature(
        &self,
        message: &str,
        signature: &Signature,
        expected_address: Address,
    ) -> bool {
        signature
            .verify(message, expected_address)
            .is_ok()
    }

    pub async fn get_client(&self) -> Result<Client, AuthError> {
        let user = self.current_user
            .as_ref()
            .ok_or(AuthError::NotAuthenticated)?;

        let client = SignerMiddleware::new(
            self.provider.clone(),
            user.wallet.clone(),
        );

        Ok(Arc::new(client))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventLogger;
    use ethers::core::k256::ecdsa::SigningKey;

    async fn setup_test_auth() -> (AuthManager, LocalWallet) {
        // Create test wallet
        let wallet: LocalWallet = SigningKey::random(&mut rand::thread_rng()).into();
        let private_key = wallet.signer().to_bytes().to_vec();
        let private_key_hex = hex::encode(private_key);

        // Create event logger
        let event_logger = Arc::new(EventLogger::new(true, false, None));

        // Create auth manager
        let mut auth = AuthManager::new("http://localhost:8545", event_logger)
            .await
            .unwrap();

        // Connect wallet
        auth.connect_wallet(&private_key_hex).await.unwrap();

        (auth, wallet)
    }

    #[tokio::test]
    async fn test_wallet_connection() {
        let (auth, wallet) = setup_test_auth().await;
        assert_eq!(auth.get_current_address().unwrap(), wallet.address());
    }

    #[tokio::test]
    async fn test_message_signing() {
        let (auth, wallet) = setup_test_auth().await;

        // Create and sign message
        let (message, signature) = auth.create_signed_request("test_action")
            .await
            .unwrap();

        // Verify signature
        let message_str = serde_json::to_string(&message).unwrap();
        assert!(auth.verify_signature(
            &message_str,
            &signature,
            wallet.address(),
        ));
    }
} 
use ethers::types::Address;
use thiserror::Error;
use std::sync::RwLock;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid wallet")]
    InvalidWallet,
    #[error("Not connected")]
    NotConnected,
}

#[derive(Debug)]
pub struct AuthManager {
    current_address: RwLock<Option<Address>>,
}

impl AuthManager {
    pub async fn new(_rpc_url: &str) -> Result<Self, AuthError> {
        Ok(Self {
            current_address: RwLock::new(None),
        })
    }

    pub async fn connect_wallet(&self, _private_key: &str) -> Result<(), AuthError> {
        // In a real implementation, this would validate the private key and set up the wallet
        // For now, we'll just set a dummy address
        *self.current_address.write().unwrap() = Some(Address::zero());
        Ok(())
    }

    pub fn get_current_address(&self) -> Result<Address, AuthError> {
        self.current_address
            .read()
            .unwrap()
            .ok_or(AuthError::NotConnected)
    }

    pub async fn create_signed_request(&self, _action: &str) -> Result<(String, String), AuthError> {
        // In a real implementation, this would create and sign a request
        // For now, we'll just return dummy values
        Ok(("dummy_message".to_string(), "dummy_signature".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_manager() {
        let auth_manager = AuthManager::new("http://localhost:8545").await.unwrap();

        // Test initial state
        assert!(auth_manager.get_current_address().is_err());

        // Test connecting wallet
        auth_manager.connect_wallet("dummy_private_key").await.unwrap();
        assert!(auth_manager.get_current_address().is_ok());

        // Test creating signed request
        let result = auth_manager.create_signed_request("test_action").await;
        assert!(result.is_ok());
    }
} 
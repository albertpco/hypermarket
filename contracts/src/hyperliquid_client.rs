use crate::{
    auth::{AuthManager, AuthError},
    MarketError,
};
use async_trait::async_trait;
use ethers::types::U256;
use hyperliquid_rust::{
    types::{
        order::{Order, Side},
        Asset, AssetInfo, L2Book, MarketMeta, Position,
    },
    HyperliquidApi, HyperliquidError,
};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HyperliquidClientError {
    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),
    #[error("Hyperliquid error: {0}")]
    HyperliquidError(#[from] HyperliquidError),
}

#[async_trait]
pub trait HyperliquidInterface {
    async fn create_market_pair(
        &self,
        market_id: &str,
        collateral_token: &str,
    ) -> Result<(String, String), HyperliquidClientError>;

    async fn place_order(&self, order: Order) -> Result<(), HyperliquidClientError>;

    async fn deposit_collateral(
        &self,
        collateral_token: &str,
        amount: U256,
    ) -> Result<(), HyperliquidClientError>;

    async fn withdraw_collateral(
        &self,
        collateral_token: &str,
        amount: U256,
    ) -> Result<(), HyperliquidClientError>;

    async fn cancel_all_orders(&self, market_address: &str) -> Result<(), HyperliquidClientError>;

    async fn settle_market(
        &self,
        winning_token_address: &str,
        winning_amount: U256,
    ) -> Result<(), HyperliquidClientError>;

    async fn get_l2_book(&self, market_address: &str) -> Result<L2Book, HyperliquidClientError>;

    async fn get_user_position(
        &self,
        user_address: &str,
        market_address: &str,
    ) -> Result<Position, HyperliquidClientError>;
}

#[derive(Clone)]
pub struct HyperliquidClient {
    api: HyperliquidApi,
    auth_manager: Arc<AuthManager>,
}

impl HyperliquidClient {
    pub fn new(api: HyperliquidApi, auth_manager: Arc<AuthManager>) -> Self {
        Self { api, auth_manager }
    }

    async fn create_signed_request(&self, action: &str) -> Result<String, HyperliquidClientError> {
        let (message, signature) = self.auth_manager
            .create_signed_request(action)
            .await
            .map_err(HyperliquidClientError::AuthError)?;

        let request = serde_json::json!({
            "message": message,
            "signature": signature,
        });

        Ok(serde_json::to_string(&request)
            .map_err(|e| HyperliquidClientError::HyperliquidError(HyperliquidError::SerializationError(e.to_string())))?)
    }
}

#[async_trait]
impl HyperliquidInterface for HyperliquidClient {
    async fn create_market_pair(
        &self,
        market_id: &str,
        collateral_token: &str,
    ) -> Result<(String, String), HyperliquidClientError> {
        let yes_token_address = format!("{}YES", market_id);
        let no_token_address = format!("{}NO", market_id);

        let asset_info = AssetInfo {
            decimals: 18,
            min_size: U256::from(1),
            min_tick: U256::from(1),
            base_maintenance_margin: U256::from(10),
            base_initial_margin: U256::from(20),
            ..Default::default()
        };

        let yes_asset = Asset {
            coin: yes_token_address.clone(),
            info: asset_info.clone(),
        };

        let no_asset = Asset {
            coin: no_token_address.clone(),
            info: asset_info,
        };

        let market_meta = MarketMeta {
            name: format!("{} YES", market_id),
            asset: yes_asset,
            collateral_asset: Asset {
                coin: collateral_token.to_string(),
                info: AssetInfo {
                    decimals: 18,
                    min_size: U256::from(1),
                    min_tick: U256::from(1),
                    ..Default::default()
                },
            },
        };

        let signed_request = self.create_signed_request("create_market_pair").await?;

        self.api
            .create_market_with_auth(&signed_request, market_meta)
            .await
            .map_err(HyperliquidClientError::HyperliquidError)?;

        let market_meta = MarketMeta {
            name: format!("{} NO", market_id),
            asset: no_asset,
            collateral_asset: Asset {
                coin: collateral_token.to_string(),
                info: AssetInfo {
                    decimals: 18,
                    min_size: U256::from(1),
                    min_tick: U256::from(1),
                    ..Default::default()
                },
            },
        };

        self.api
            .create_market_with_auth(&signed_request, market_meta)
            .await
            .map_err(HyperliquidClientError::HyperliquidError)?;

        Ok((yes_token_address, no_token_address))
    }

    async fn place_order(&self, order: Order) -> Result<(), HyperliquidClientError> {
        let signed_request = self.create_signed_request("place_order").await?;

        self.api
            .place_order_with_auth(&signed_request, order)
            .await
            .map_err(HyperliquidClientError::HyperliquidError)?;

        Ok(())
    }

    async fn deposit_collateral(
        &self,
        collateral_token: &str,
        amount: U256,
    ) -> Result<(), HyperliquidClientError> {
        let signed_request = self.create_signed_request("deposit_collateral").await?;

        self.api
            .deposit_collateral_with_auth(&signed_request, collateral_token, amount)
            .await
            .map_err(HyperliquidClientError::HyperliquidError)?;

        Ok(())
    }

    async fn withdraw_collateral(
        &self,
        collateral_token: &str,
        amount: U256,
    ) -> Result<(), HyperliquidClientError> {
        let signed_request = self.create_signed_request("withdraw_collateral").await?;

        self.api
            .withdraw_collateral_with_auth(&signed_request, collateral_token, amount)
            .await
            .map_err(HyperliquidClientError::HyperliquidError)?;

        Ok(())
    }

    async fn cancel_all_orders(&self, market_address: &str) -> Result<(), HyperliquidClientError> {
        let signed_request = self.create_signed_request("cancel_all_orders").await?;

        self.api
            .cancel_all_orders_with_auth(&signed_request, market_address)
            .await
            .map_err(HyperliquidClientError::HyperliquidError)?;

        Ok(())
    }

    async fn settle_market(
        &self,
        winning_token_address: &str,
        winning_amount: U256,
    ) -> Result<(), HyperliquidClientError> {
        let signed_request = self.create_signed_request("settle_market").await?;

        self.api
            .settle_market_with_auth(&signed_request, winning_token_address, winning_amount)
            .await
            .map_err(HyperliquidClientError::HyperliquidError)?;

        Ok(())
    }

    async fn get_l2_book(&self, market_address: &str) -> Result<L2Book, HyperliquidClientError> {
        self.api
            .get_l2_book(market_address)
            .await
            .map_err(HyperliquidClientError::HyperliquidError)
    }

    async fn get_user_position(
        &self,
        user_address: &str,
        market_address: &str,
    ) -> Result<Position, HyperliquidClientError> {
        self.api
            .get_user_position(user_address, market_address)
            .await
            .map_err(HyperliquidClientError::HyperliquidError)
    }
} 
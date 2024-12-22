use crate::{
    auth::{AuthManager, AuthError},
    events::{EventEmitter, MarketEvent},
    Market, MarketContract, MarketStatus,
};
use async_trait::async_trait;
use ethers::types::{Address, U256};
use hyperliquid_rust::{
    types::{
        order::{Order, OrderType, Side},
        Asset, AssetInfo, L2Book, Position,
    },
    HyperliquidApi, HyperliquidError,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarketError {
    #[error("Market is not active")]
    MarketNotActive,
    #[error("Market is not expired")]
    MarketNotExpired,
    #[error("Market is not resolved")]
    MarketNotResolved,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Invalid order")]
    InvalidOrder,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Market already resolved")]
    MarketAlreadyResolved,
    #[error("Invalid oracle")]
    InvalidOracle,
    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),
    #[error("Hyperliquid error: {0}")]
    HyperliquidError(#[from] HyperliquidError),
    #[error("Insufficient collateral")]
    InsufficientCollateral,
    #[error("Invalid collateral token")]
    InvalidCollateralToken,
    #[error("Collateral transfer failed")]
    CollateralTransferFailed,
    #[error("Withdrawal amount exceeds available balance")]
    WithdrawalExceedsBalance,
    #[error("Order placement failed")]
    OrderPlacementFailed,
    #[error("Order cancellation failed")]
    OrderCancellationFailed,
    #[error("Market settlement failed")]
    MarketSettlementFailed,
    #[error("Invalid signature")]
    InvalidSignature,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MarketContractState {
    pub market: Market,
    pub api: HyperliquidApi,
    pub yes_token_supply: U256,
    pub no_token_supply: U256,
    pub user_balances: std::collections::HashMap<Address, (U256, U256)>, // (yes_tokens, no_tokens)
    pub collateral_balances: std::collections::HashMap<Address, U256>, // User collateral balances
    pub total_collateral: U256, // Total collateral in the market
    auth_manager: Arc<AuthManager>,
    event_emitter: Arc<dyn EventEmitter>,
    client: HyperliquidClient,
}

impl MarketContractState {
    pub async fn new(
        market: Market,
        api_url: &str,
        auth_manager: Arc<AuthManager>,
        event_emitter: Arc<dyn EventEmitter>,
    ) -> Result<Self, MarketError> {
        let api = HyperliquidApi::new(api_url)
            .map_err(MarketError::HyperliquidError)?;

        let client = HyperliquidClient::new(api.clone(), auth_manager.clone());

        let mut market_contract = Self {
            market,
            api,
            yes_token_supply: U256::zero(),
            no_token_supply: U256::zero(),
            user_balances: std::collections::HashMap::new(),
            collateral_balances: std::collections::HashMap::new(),
            total_collateral: U256::zero(),
            auth_manager,
            event_emitter,
            client,
        };

        // Spawn a background task to check market expiry
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = market_contract.check_expiry().await {
                    eprintln!("Error checking market expiry: {}", e);
                }
            }
        });

        Ok(market_contract)
    }

    async fn get_order_book(&self) -> Result<L2Book, MarketError> {
        // Fetch order book from Hyperliquid API
        self.client
            .get_l2_book(&self.market.yes_token_address)
            .await
            .map_err(MarketError::HyperliquidError)
    }

    async fn get_user_position(&self, user_address: Address) -> Result<Position, MarketError> {
        self.api
            .get_user_position(&format!("{:?}", user_address), &self.market.yes_token_address)
            .await
            .map_err(MarketError::HyperliquidError)
    }

    async fn get_caller_address(&self) -> Result<Address, MarketError> {
        self.auth_manager.get_current_address().map_err(MarketError::AuthError)
    }

    async fn create_signed_request(&self, action: &str) -> Result<String, MarketError> {
        let (message, signature) = self.auth_manager
            .create_signed_request(action)
            .await
            .map_err(MarketError::AuthError)?;

        let request = serde_json::json!({
            "message": message,
            "signature": signature,
        });

        Ok(serde_json::to_string(&request)
            .map_err(|e| MarketError::HyperliquidError(HyperliquidError::SerializationError(e.to_string())))?)
    }

    pub async fn deposit_collateral(&mut self, amount: U256) -> Result<(), MarketError> {
        if self.market.status != MarketStatus::Active {
            return Err(MarketError::MarketNotActive);
        }

        let user_address = self.get_caller_address().await?;
        let signed_request = self.create_signed_request("deposit_collateral").await?;

        // Process deposit on Hyperliquid
        let tx_hash = self.client
            .deposit_collateral(&self.market.collateral_token, amount)
            .await
            .map_err(|e| {
                if e.to_string().contains("transfer") {
                    MarketError::CollateralTransferFailed
                } else {
                    MarketError::HyperliquidError(e)
                }
            })?;

        // Update local state
        *self.collateral_balances.entry(user_address).or_insert(U256::zero()) += amount;
        self.total_collateral += amount;

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::CollateralDeposited {
            market_id: self.market.yes_token_address.clone(),
            user: user_address,
            amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash,
        });

        Ok(())
    }

    pub async fn withdraw_collateral(&mut self, amount: U256) -> Result<(), MarketError> {
        if self.market.status != MarketStatus::Active {
            return Err(MarketError::MarketNotActive);
        }

        let user_address = self.get_caller_address().await?;
        let balance = self.collateral_balances
            .get(&user_address)
            .copied()
            .unwrap_or(U256::zero());

        if balance < amount {
            return Err(MarketError::WithdrawalExceedsBalance);
        }

        // Check if user has any open positions or orders that require collateral
        let position = self.get_user_position(user_address).await?;
        let required_collateral = self.calculate_required_collateral(&position)?;
        
        if balance.saturating_sub(amount) < required_collateral {
            return Err(MarketError::InsufficientCollateral);
        }

        // Process withdrawal on Hyperliquid
        let tx_hash = self.client
            .withdraw_collateral(&self.market.collateral_token, amount)
            .await
            .map_err(MarketError::HyperliquidError)?;

        // Update local state
        *self.collateral_balances.get_mut(&user_address).unwrap() -= amount;
        self.total_collateral -= amount;

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::CollateralWithdrawn {
            market_id: self.market.yes_token_address.clone(),
            user: user_address,
            amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash,
        });

        Ok(())
    }

    async fn calculate_required_collateral(&self, position: &Position) -> Result<U256, MarketError> {
        // Get current order book and user's open orders
        let order_book = self.get_order_book().await?;
        
        // Calculate required collateral based on:
        // 1. Open positions
        // 2. Open orders
        // 3. Market volatility
        // 4. Position risk

        // Position collateral requirement
        let position_size = position.size.abs();
        let mark_price = if !order_book.bids.is_empty() && !order_book.asks.is_empty() {
            // Use mid price as mark price
            (order_book.bids[0].price + order_book.asks[0].price) / U256::from(2)
        } else {
            position.entry_price
        };

        // Calculate position value and required margin
        let position_value = position_size * mark_price;
        let maintenance_margin_rate = U256::from(10); // 10%
        let initial_margin_rate = U256::from(20);     // 20%
        
        // Calculate volatility multiplier based on recent price movement
        let volatility_multiplier = self.calculate_volatility_multiplier(&order_book)?;
        
        // Position margin requirement
        let position_margin = position_value * initial_margin_rate * volatility_multiplier / U256::from(100);

        // Order margin requirement
        let order_margin = order_book.bids.iter()
            .map(|level| {
                let order_value = level.size * level.price;
                order_value * initial_margin_rate / U256::from(100)
            })
            .sum::<U256>();

        // Calculate liquidation buffer
        let liquidation_buffer = if position_value > U256::zero() {
            position_value * maintenance_margin_rate / U256::from(100)
        } else {
            U256::zero()
        };

        // Total required collateral is sum of:
        // 1. Position margin
        // 2. Order margin
        // 3. Liquidation buffer
        let total_required = position_margin + order_margin + liquidation_buffer;

        // Add minimum collateral requirement
        let minimum_collateral = U256::from(100); // Minimum 100 base units
        Ok(std::cmp::max(total_required, minimum_collateral))
    }

    fn calculate_volatility_multiplier(&self, order_book: &L2Book) -> Result<U256, MarketError> {
        if order_book.bids.is_empty() || order_book.asks.is_empty() {
            return Ok(U256::from(100)); // Default 1.0x multiplier
        }

        // Calculate price spread
        let best_bid = order_book.bids[0].price;
        let best_ask = order_book.asks[0].price;
        let mid_price = (best_bid + best_ask) / U256::from(2);
        
        if mid_price == U256::zero() {
            return Ok(U256::from(100));
        }

        // Calculate spread percentage
        let spread_bps = (best_ask - best_bid) * U256::from(10000) / mid_price;
        
        // Adjust multiplier based on spread
        // <50 bps: 1.0x
        // 50-100 bps: 1.2x
        // 100-200 bps: 1.5x
        // >200 bps: 2.0x
        let multiplier = if spread_bps < U256::from(50) {
            U256::from(100)
        } else if spread_bps < U256::from(100) {
            U256::from(120)
        } else if spread_bps < U256::from(200) {
            U256::from(150)
        } else {
            U256::from(200)
        };

        Ok(multiplier)
    }

    pub async fn get_collateral_balance(&self, user: Address) -> Result<U256, MarketError> {
        Ok(self.collateral_balances.get(&user).copied().unwrap_or(U256::zero()))
    }

    async fn place_order(&mut self, side: Side, price: U256, amount: U256) -> Result<(), MarketError> {
        if self.market.status != MarketStatus::Active {
            return Err(MarketError::MarketNotActive);
        }

        let user_address = self.get_caller_address().await?;
        let balance = self.user_balances.get(&user_address).unwrap_or(&(U256::zero(), U256::zero()));

        // Check if user has sufficient balance
        match side {
            Side::Buy => {
                if balance.1 < amount {
                    return Err(MarketError::InsufficientBalance);
                }
            }
            Side::Sell => {
                if balance.0 < amount {
                    return Err(MarketError::InsufficientBalance);
                }
            }
        }

        // Calculate required collateral for the order
        let required_collateral = price * amount;
        let user_collateral = self.collateral_balances
            .get(&user_address)
            .copied()
            .unwrap_or(U256::zero());

        // Check if user has sufficient collateral for the order
        let position = self.get_user_position(user_address).await?;
        let current_required_collateral = self.calculate_required_collateral(&position)?;
        let total_required = current_required_collateral + required_collateral;

        if user_collateral < total_required {
            return Err(MarketError::InsufficientCollateral);
        }

        // Get current order book to check liquidity
        let order_book = self.get_order_book().await?;
        
        // Validate order against current market conditions
        self.validate_order(&order_book, side, price, amount)?;

        // Create and submit order to Hyperliquid
        let order = Order {
            asset: Asset {
                coin: self.market.yes_token_address.clone(),
                info: AssetInfo {
                    decimals: 18,
                    min_size: U256::from(1),
                    min_tick: U256::from(1),
                    base_maintenance_margin: U256::from(10), // 10% maintenance margin
                    base_initial_margin: U256::from(20),     // 20% initial margin
                    ..Default::default()
                },
            },
            side,
            order_type: OrderType::Limit,
            price,
            amount,
            reduce_only: false,
        };

        // Submit order to Hyperliquid
        let tx_hash = self.client
            .place_order(order)
            .await
            .map_err(|e| {
                MarketError::OrderPlacementFailed
            })?;

        // Lock additional collateral
        *self.collateral_balances.get_mut(&user_address).unwrap() -= required_collateral;

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::OrderPlaced {
            market_id: self.market.yes_token_address.clone(),
            user: user_address,
            side: format!("{:?}", side),
            price,
            amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash,
        });

        Ok(())
    }

    fn validate_order(&self, order_book: &L2Book, side: Side, price: U256, amount: U256) -> Result<(), MarketError> {
        // Validate minimum order size
        if amount < U256::from(1) {
            return Err(MarketError::InvalidAmount);
        }

        // Validate price against current market
        match side {
            Side::Buy => {
                // For buy orders, check if price is not too high above best ask
                if !order_book.asks.is_empty() {
                    let best_ask = order_book.asks[0].price;
                    if price > best_ask * U256::from(110) / U256::from(100) { // Max 10% above best ask
                        return Err(MarketError::InvalidOrder);
                    }
                }
            }
            Side::Sell => {
                // For sell orders, check if price is not too low below best bid
                if !order_book.bids.is_empty() {
                    let best_bid = order_book.bids[0].price;
                    if price < best_bid * U256::from(90) / U256::from(100) { // Max 10% below best bid
                        return Err(MarketError::InvalidOrder);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn mint_tokens(&mut self, amount: U256) -> Result<(), MarketError> {
        if self.market.status != MarketStatus::Active {
            return Err(MarketError::MarketNotActive);
        }

        let user_address = self.get_caller_address().await?;

        // Update token supply
        self.yes_token_supply += amount;
        self.no_token_supply += amount;

        // Update user balances
        let user_balance = self.user_balances
            .entry(user_address)
            .or_insert((U256::zero(), U256::zero()));
        user_balance.0 += amount; // yes tokens
        user_balance.1 += amount; // no tokens

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::TokensMinted {
            market_id: self.market.yes_token_address.clone(),
            user: user_address,
            amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        Ok(())
    }

    pub async fn burn_tokens(&mut self, yes_amount: U256, no_amount: U256) -> Result<(), MarketError> {
        if self.market.status != MarketStatus::Active {
            return Err(MarketError::MarketNotActive);
        }

        let user_address = self.get_caller_address().await?;
        let balance = self.user_balances
            .get(&user_address)
            .ok_or(MarketError::InsufficientBalance)?;

        // Check if user has sufficient balance
        if balance.0 < yes_amount || balance.1 < no_amount {
            return Err(MarketError::InsufficientBalance);
        }

        let signed_request = self.create_signed_request("burn_tokens").await?;

        // Process burn transaction on Hyperliquid
        let tx_hash = self.client
            .burn_tokens_with_auth(&signed_request, &self.market.yes_token_address, yes_amount, no_amount)
            .await
            .map_err(MarketError::HyperliquidError)?;

        // Update token supply
        self.yes_token_supply -= yes_amount;
        self.no_token_supply -= no_amount;

        // Update user balances
        let user_balance = self.user_balances.get_mut(&user_address).unwrap();
        user_balance.0 -= yes_amount;
        user_balance.1 -= no_amount;

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::TokensBurned {
            market_id: self.market.yes_token_address.clone(),
            user: user_address,
            yes_amount,
            no_amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash,
        });

        Ok(())
    }

    pub async fn resolve(&mut self) -> Result<(), MarketError> {
        if self.market.status != MarketStatus::Expired {
            return Err(MarketError::MarketNotExpired);
        }

        let user_address = self.get_caller_address().await?;
        
        // Verify caller is the designated oracle
        if format!("{:?}", user_address) != self.market.oracle_id {
            return Err(MarketError::InvalidOracle);
        }

        if self.market.resolved_outcome.is_some() {
            return Err(MarketError::MarketAlreadyResolved);
        }

        // Fetch outcome from OracleManager
        let outcome = self.get_oracle_outcome().await?;

        // Cancel all open orders
        let tx_hash = self.client
            .cancel_all_orders(&self.market.yes_token_address)
            .await
            .map_err(MarketError::HyperliquidError)?;

        // Update market status
        self.set_market_status(MarketStatus::Resolved);
        self.market.resolved_outcome = Some(outcome);

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::MarketResolved {
            market_id: self.market.yes_token_address.clone(),
            oracle: user_address,
            outcome,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash,
        });

        Ok(())
    }

    async fn get_oracle_outcome(&self) -> Result<bool, MarketError> {
        // Get outcome from OracleManager
        let oracle_manager = self.get_oracle_manager().await?;
        oracle_manager
            .get_outcome(self.market.yes_token_address.clone())
            .ok_or(MarketError::MarketNotResolved)
    }

    async fn get_oracle_manager(&self) -> Result<Arc<dyn OracleManager>, MarketError> {
        // Get OracleManager from context
        // This is a placeholder, in a real application, you would need to get the OracleManager from a context or dependency injection
        // For now, we will create a new OracleManager
        let auth_manager = self.auth_manager.clone();
        let event_emitter = self.event_emitter.clone();
        let oracle_manager = OracleManagerState::new(auth_manager, event_emitter);
        Ok(Arc::new(oracle_manager))
    }

    fn set_market_status(&mut self, status: MarketStatus) {
        self.market.status = status;
    }

    pub async fn claim_winnings(&mut self) -> Result<U256, MarketError> {
        if self.market.status != MarketStatus::Resolved {
            return Err(MarketError::MarketNotResolved);
        }

        let user_address = self.get_caller_address().await?;
        let balance = self.user_balances
            .get(&user_address)
            .ok_or(MarketError::InsufficientBalance)?;

        let outcome = self.market.resolved_outcome.unwrap();
        let winning_token_address = if outcome {
            &self.market.yes_token_address
        } else {
            &self.market.no_token_address
        };

        let winning_amount = if outcome {
            balance.0 // YES tokens
        } else {
            balance.1 // NO tokens
        };

        if winning_amount == U256::zero() {
            return Err(MarketError::InsufficientBalance);
        }

        // Get user's position from Hyperliquid
        let position = self.get_user_position(user_address).await?;
        
        // Cancel any remaining open orders
        let tx_hash = self.client
            .cancel_all_orders(winning_token_address)
            .await
            .map_err(MarketError::HyperliquidError)?;

        // Calculate settlement price based on outcome
        let settlement_price = if outcome {
            U256::from(1) // YES tokens worth 1 collateral token
        } else {
            U256::from(0) // YES tokens worth 0 collateral token
        };

        // Process settlement on Hyperliquid
        let tx_hash = self.client
            .settle_market(winning_token_address, winning_amount)
            .await
            .map_err(|_| MarketError::MarketSettlementFailed)?;

        // Calculate collateral to return
        let collateral_to_return = winning_amount * settlement_price;
        
        // Update collateral balance
        if collateral_to_return > U256::zero() {
            *self.collateral_balances.entry(user_address).or_insert(U256::zero()) += collateral_to_return;
        }

        // Clear user's token balance
        self.user_balances.remove(&user_address);

        // Update total supply
        self.yes_token_supply -= balance.0;
        self.no_token_supply -= balance.1;

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::WinningsClaimed {
            market_id: self.market.yes_token_address.clone(),
            user: user_address,
            amount: winning_amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash,
        });

        Ok(winning_amount)
    }

    pub async fn cancel_order(&mut self, order_id: String) -> Result<(), MarketError> {
        if self.market.status != MarketStatus::Active {
            return Err(MarketError::MarketNotActive);
        }

        let user_address = self.get_caller_address().await?;

        // Cancel order on Hyperliquid
        // Note: Hyperliquid API does not support cancelling a single order by ID
        // We will cancel all orders for now
        let tx_hash = self.client
            .cancel_all_orders(&self.market.yes_token_address)
            .await
            .map_err(|_| MarketError::OrderCancellationFailed)?;

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::OrderCancelled {
            market_id: self.market.yes_token_address.clone(),
            user: user_address,
            order_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash,
        });

        Ok(())
    }

    pub async fn cancel_all_user_orders(&mut self) -> Result<(), MarketError> {
        if self.market.status != MarketStatus::Active {
            return Err(MarketError::MarketNotActive);
        }

        let user_address = self.get_caller_address().await?;

        // Cancel all user orders on Hyperliquid
        self.client
            .cancel_all_orders(&self.market.yes_token_address)
            .await
            .map_err(MarketError::HyperliquidError)?;

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::OrdersCancelled {
            market_id: self.market.yes_token_address.clone(),
            user: user_address,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        Ok(())
    }

    pub async fn check_expiry(&mut self) -> Result<(), MarketError> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if current_time >= self.market.expiry_timestamp && self.market.status == MarketStatus::Active {
            self.set_market_status(MarketStatus::Expired);

            // Emit event
            let user_address = self.get_caller_address().await?;
            self.event_emitter.emit_market_event(MarketEvent::MarketExpired {
                market_id: self.market.yes_token_address.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }

        Ok(())
    }
}

#[async_trait]
impl MarketContract for MarketContractState {
    async fn mint_tokens(&mut self, amount: U256) -> Result<(), MarketError> {
        if self.market.status != MarketStatus::Active {
            return Err(MarketError::MarketNotActive);
        }

        let user_address = self.get_caller_address().await?;
        
        // Check if user has sufficient collateral
        let collateral_required = amount; // 1:1 ratio for simplicity
        let user_collateral = self.collateral_balances
            .get(&user_address)
            .copied()
            .unwrap_or(U256::zero());

        if user_collateral < collateral_required {
            return Err(MarketError::InsufficientCollateral);
        }

        let signed_request = self.create_signed_request("mint_tokens").await?;

        // Create mint transaction on Hyperliquid
        self.api
            .mint_tokens_with_auth(
                &signed_request,
                &self.market.yes_token_address,
                &self.market.no_token_address,
                amount,
            )
            .await
            .map_err(MarketError::HyperliquidError)?;

        // Place orders on Hyperliquid for both YES and NO tokens
        // YES token order
        let yes_order = Order {
            asset: Asset {
                coin: self.market.yes_token_address.clone(),
                info: AssetInfo {
                    decimals: 18,
                    min_size: U256::from(1),
                    min_tick: U256::from(1),
                    ..Default::default()
                },
            },
            side: Side::Buy,
            order_type: OrderType::Market, // Use market order for immediate execution
            price: U256::from(1), // 1:1 ratio with collateral
            amount,
            reduce_only: false,
        };

        // NO token order
        let no_order = Order {
            asset: Asset {
                coin: self.market.no_token_address.clone(),
                info: AssetInfo {
                    decimals: 18,
                    min_size: U256::from(1),
                    min_tick: U256::from(1),
                    ..Default::default()
                },
            },
            side: Side::Buy,
            order_type: OrderType::Market,
            price: U256::from(1),
            amount,
            reduce_only: false,
        };

        // Submit orders to Hyperliquid
        self.api
            .place_order_with_auth(&signed_request, yes_order)
            .await
            .map_err(MarketError::HyperliquidError)?;

        self.api
            .place_order_with_auth(&signed_request, no_order)
            .await
            .map_err(MarketError::HyperliquidError)?;

        // Lock collateral
        *self.collateral_balances.get_mut(&user_address).unwrap() -= collateral_required;

        // Update local state
        self.yes_token_supply += amount;
        self.no_token_supply += amount;

        let balance = self.user_balances.entry(user_address).or_insert((U256::zero(), U256::zero()));
        balance.0 += amount;
        balance.1 += amount;

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::TokensMinted {
            market_id: self.market.yes_token_address.clone(),
            user: user_address,
            amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        Ok(())
    }

    async fn resolve(&mut self) -> Result<(), MarketError> {
        if self.market.status != MarketStatus::Expired {
            return Err(MarketError::MarketNotExpired);
        }

        let user_address = self.get_caller_address().await?;
        
        // Verify caller is the designated oracle
        if format!("{:?}", user_address) != self.market.oracle_id {
            return Err(MarketError::InvalidOracle);
        }

        if self.market.resolved_outcome.is_some() {
            return Err(MarketError::MarketAlreadyResolved);
        }

        // Fetch outcome from OracleManager
        let outcome = self.get_oracle_outcome().await?;

        // Cancel all open orders
        let tx_hash = self.client
            .cancel_all_orders(&self.market.yes_token_address)
            .await
            .map_err(MarketError::HyperliquidError)?;

        // Update market status
        self.set_market_status(MarketStatus::Resolved);
        self.market.resolved_outcome = Some(outcome);

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::MarketResolved {
            market_id: self.market.yes_token_address.clone(),
            oracle: user_address,
            outcome,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash,
        });

        Ok(())
    }

    async fn claim_winnings(&mut self) -> Result<U256, MarketError> {
        if self.market.status != MarketStatus::Resolved {
            return Err(MarketError::MarketNotResolved);
        }

        let user_address = self.get_caller_address().await?;
        let balance = self.user_balances
            .get(&user_address)
            .ok_or(MarketError::InsufficientBalance)?;

        let outcome = self.market.resolved_outcome.unwrap();
        let winning_token_address = if outcome {
            &self.market.yes_token_address
        } else {
            &self.market.no_token_address
        };

        let winning_amount = if outcome {
            balance.0 // YES tokens
        } else {
            balance.1 // NO tokens
        };

        if winning_amount == U256::zero() {
            return Err(MarketError::InsufficientBalance);
        }

        // Get user's position from Hyperliquid
        let position = self.get_user_position(user_address).await?;
        
        // Cancel any remaining open orders
        let tx_hash = self.client
            .cancel_all_orders(winning_token_address)
            .await
            .map_err(MarketError::HyperliquidError)?;

        // Calculate settlement price based on outcome
        let settlement_price = if outcome {
            U256::from(1) // YES tokens worth 1 collateral token
        } else {
            U256::from(0) // YES tokens worth 0 collateral token
        };

        // Process settlement on Hyperliquid
        let tx_hash = self.client
            .settle_market(winning_token_address, winning_amount)
            .await
            .map_err(|_| MarketError::MarketSettlementFailed)?;

        // Calculate collateral to return
        let collateral_to_return = winning_amount * settlement_price;
        
        // Update collateral balance
        if collateral_to_return > U256::zero() {
            *self.collateral_balances.entry(user_address).or_insert(U256::zero()) += collateral_to_return;
        }

        // Clear user's token balance
        self.user_balances.remove(&user_address);

        // Update total supply
        self.yes_token_supply -= balance.0;
        self.no_token_supply -= balance.1;

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::WinningsClaimed {
            market_id: self.market.yes_token_address.clone(),
            user: user_address,
            amount: winning_amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash,
        });

        Ok(winning_amount)
    }

    async fn burn_tokens(&mut self, yes_amount: U256, no_amount: U256) -> Result<(), MarketError> {
        if self.market.status != MarketStatus::Active {
            return Err(MarketError::MarketNotActive);
        }

        let user_address = self.get_caller_address().await?;
        let balance = self.user_balances
            .get(&user_address)
            .ok_or(MarketError::InsufficientBalance)?;

        // Check if user has sufficient balance
        if balance.0 < yes_amount || balance.1 < no_amount {
            return Err(MarketError::InsufficientBalance);
        }

        let signed_request = self.create_signed_request("burn_tokens").await?;

        // Process burn transaction on Hyperliquid
        let tx_hash = self.client
            .burn_tokens_with_auth(&signed_request, &self.market.yes_token_address, yes_amount, no_amount)
            .await
            .map_err(MarketError::HyperliquidError)?;

        // Update token supply
        self.yes_token_supply -= yes_amount;
        self.no_token_supply -= no_amount;

        // Update user balances
        let user_balance = self.user_balances.get_mut(&user_address).unwrap();
        user_balance.0 -= yes_amount;
        user_balance.1 -= no_amount;

        // Emit event
        self.event_emitter.emit_market_event(MarketEvent::TokensBurned {
            market_id: self.market.yes_token_address.clone(),
            user: user_address,
            yes_amount,
            no_amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash,
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

    async fn setup_test_market() -> (MarketContractState, LocalWallet) {
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

        // Create test market
        let market = Market {
            question: "Test Market?".to_string(),
            expiry_timestamp: 0,
            oracle_id: "test_oracle".to_string(),
            collateral_token: "USDC".to_string(),
            status: MarketStatus::Active,
            yes_token_address: "YES_TOKEN".to_string(),
            no_token_address: "NO_TOKEN".to_string(),
            resolved_outcome: None,
        };

        let market_contract = MarketContractState::new(
            market,
            "http://localhost:8080",
            auth_manager,
            event_logger,
        )
        .await
        .unwrap();

        (market_contract, wallet)
    }

    #[tokio::test]
    async fn test_mint_tokens() {
        let (mut market_contract, _) = setup_test_market().await;
        let result = market_contract.mint_tokens(U256::from(100)).await;
        assert!(result.is_ok());
        assert_eq!(market_contract.yes_token_supply, U256::from(100));
        assert_eq!(market_contract.no_token_supply, U256::from(100));
        let user_address = market_contract.get_caller_address().await.unwrap();
        assert_eq!(market_contract.user_balances.get(&user_address).unwrap(), &(U256::from(100), U256::from(100)));
    }

    #[tokio::test]
    async fn test_place_order() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // First mint some tokens
        market_contract.mint_tokens(U256::from(100)).await.unwrap();

        // Then place an order
        let result = market_contract
            .place_order(Side::Buy, U256::from(50), U256::from(10))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resolve_market() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Set market to expired
        market_contract.market.status = MarketStatus::Expired;

        // Resolve market
        let result = market_contract.resolve().await;
        assert!(result.is_ok());
        assert_eq!(market_contract.market.status, MarketStatus::Resolved);
        assert_eq!(market_contract.market.resolved_outcome, Some(true));
    }

    #[tokio::test]
    async fn test_burn_tokens() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Mint some tokens
        market_contract.mint_tokens(U256::from(100)).await.unwrap();

        // Burn some tokens
        let result = market_contract
            .burn_tokens(U256::from(30), U256::from(40))
            .await;
        assert!(result.is_ok());

        // Verify balances
        let user_address = market_contract.get_caller_address().await.unwrap();
        let balance = market_contract.user_balances.get(&user_address).unwrap();
        assert_eq!(balance.0, U256::from(70)); // YES tokens: 100 - 30
        assert_eq!(balance.1, U256::from(60)); // NO tokens: 100 - 40
    }

    #[tokio::test]
    async fn test_burn_tokens_insufficient_balance() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Mint fewer tokens than we try to burn
        market_contract.mint_tokens(U256::from(50)).await.unwrap();

        // Try to burn more tokens than available
        let result = market_contract
            .burn_tokens(U256::from(60), U256::from(40))
            .await;
        assert!(matches!(result, Err(MarketError::InsufficientBalance)));
    }

    #[tokio::test]
    async fn test_market_inactive() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Set market to resolved status
        market_contract.market.status = MarketStatus::Resolved;

        // Try to mint tokens
        let result = market_contract.mint_tokens(U256::from(100)).await;
        assert!(matches!(result, Err(MarketError::MarketNotActive)));

        // Try to burn tokens
        let result = market_contract
            .burn_tokens(U256::from(50), U256::from(40))
            .await;
        assert!(matches!(result, Err(MarketError::MarketNotActive)));

        // Try to place order
        let result = market_contract
            .place_order(Side::Buy, U256::from(50), U256::from(10))
            .await;
        assert!(matches!(result, Err(MarketError::MarketNotActive)));
    }

    #[tokio::test]
    async fn test_claim_winnings_not_resolved() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Try to claim winnings before market is resolved
        let result = market_contract.claim_winnings().await;
        assert!(matches!(result, Err(MarketError::MarketNotResolved)));
    }

    #[tokio::test]
    async fn test_resolve_not_expired() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Try to resolve market before it's expired
        let result = market_contract.resolve().await;
        assert!(matches!(result, Err(MarketError::MarketNotExpired)));
    }

    #[tokio::test]
    async fn test_resolve_invalid_oracle() {
        let (mut market_contract, wallet) = setup_test_market().await;
        
        // Set market to expired status
        market_contract.market.status = MarketStatus::Expired;

        // Set a different oracle ID than the caller's address
        market_contract.market.oracle_id = "different_oracle".to_string();

        // Try to resolve market with non-oracle address
        let result = market_contract.resolve().await;
        assert!(matches!(result, Err(MarketError::InvalidOracle)));
    }

    #[tokio::test]
    async fn test_resolve_already_resolved() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Set market to expired and set oracle ID to match caller
        market_contract.market.status = MarketStatus::Expired;
        let caller = market_contract.get_caller_address().await.unwrap();
        market_contract.market.oracle_id = format!("{:?}", caller);

        // Submit outcome to oracle
        let oracle_manager = market_contract.get_oracle_manager().await.unwrap();
        oracle_manager.submit_outcome(market_contract.market.yes_token_address.clone(), true).await.unwrap();

        // Resolve market first time
        market_contract.resolve().await.unwrap();

        // Try to resolve again
        let result = market_contract.resolve().await;
        assert!(matches!(result, Err(MarketError::MarketAlreadyResolved)));
    }

    #[tokio::test]
    async fn test_deposit_collateral() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Deposit collateral
        let result = market_contract.deposit_collateral(U256::from(1000)).await;
        assert!(result.is_ok());

        // Verify balance
        let user_address = market_contract.get_caller_address().await.unwrap();
        let balance = market_contract.get_collateral_balance(user_address).await.unwrap();
        assert_eq!(balance, U256::from(1000));
        assert_eq!(market_contract.total_collateral, U256::from(1000));
    }

    #[tokio::test]
    async fn test_withdraw_collateral() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // First deposit some collateral
        market_contract.deposit_collateral(U256::from(1000)).await.unwrap();

        // Then withdraw part of it
        let result = market_contract.withdraw_collateral(U256::from(400)).await;
        assert!(result.is_ok());

        // Verify balance
        let user_address = market_contract.get_caller_address().await.unwrap();
        let balance = market_contract.get_collateral_balance(user_address).await.unwrap();
        assert_eq!(balance, U256::from(600));
        assert_eq!(market_contract.total_collateral, U256::from(600));
    }

    #[tokio::test]
    async fn test_withdraw_too_much_collateral() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Deposit some collateral
        market_contract.deposit_collateral(U256::from(1000)).await.unwrap();

        // Try to withdraw more than deposited
        let result = market_contract.withdraw_collateral(U256::from(1500)).await;
        assert!(matches!(result, Err(MarketError::WithdrawalExceedsBalance)));
    }

    #[tokio::test]
    async fn test_withdraw_with_open_position() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Deposit collateral
        market_contract.deposit_collateral(U256::from(1000)).await.unwrap();

        // Mint tokens and place an order to create a position
        market_contract.mint_tokens(U256::from(100)).await.unwrap();
        market_contract.place_order(Side::Buy, U256::from(50), U256::from(10)).await.unwrap();

        // Try to withdraw all collateral
        let result = market_contract.withdraw_collateral(U256::from(1000)).await;
        assert!(matches!(result, Err(MarketError::InsufficientCollateral)));
    }

    #[tokio::test]
    async fn test_collateral_inactive_market() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Set market to resolved status
        market_contract.market.status = MarketStatus::Resolved;

        // Try to deposit collateral
        let result = market_contract.deposit_collateral(U256::from(1000)).await;
        assert!(matches!(result, Err(MarketError::MarketNotActive)));

        // Try to withdraw collateral
        let result = market_contract.withdraw_collateral(U256::from(1000)).await;
        assert!(matches!(result, Err(MarketError::MarketNotActive)));
    }

    #[tokio::test]
    async fn test_claim_winnings() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Mint some tokens
        market_contract.mint_tokens(U256::from(100)).await.unwrap();

        // Set market to expired and resolved
        market_contract.market.status = MarketStatus::Expired;
        let caller = market_contract.get_caller_address().await.unwrap();
        market_contract.market.oracle_id = format!("{:?}", caller);

        // Submit outcome to oracle
        let oracle_manager = market_contract.get_oracle_manager().await.unwrap();
        oracle_manager.submit_outcome(market_contract.market.yes_token_address.clone(), true).await.unwrap();

        // Resolve market
        market_contract.resolve().await.unwrap();

        // Claim winnings
        let result = market_contract.claim_winnings().await;
        assert!(result.is_ok());

        // Verify user balance is cleared
        let user_address = market_contract.get_caller_address().await.unwrap();
        assert!(market_contract.user_balances.get(&user_address).is_none());
    }

    #[tokio::test]
    async fn test_claim_winnings_not_resolved() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Try to claim winnings before market is resolved
        let result = market_contract.claim_winnings().await;
        assert!(matches!(result, Err(MarketError::MarketNotResolved)));
    }

    #[tokio::test]
    async fn test_resolve_not_expired() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Try to resolve market before it's expired
        let result = market_contract.resolve().await;
        assert!(matches!(result, Err(MarketError::MarketNotExpired)));
    }

    #[tokio::test]
    async fn test_resolve_invalid_oracle() {
        let (mut market_contract, wallet) = setup_test_market().await;
        
        // Set market to expired status
        market_contract.market.status = MarketStatus::Expired;

        // Set a different oracle ID than the caller's address
        market_contract.market.oracle_id = "different_oracle".to_string();

        // Try to resolve market with non-oracle address
        let result = market_contract.resolve().await;
        assert!(matches!(result, Err(MarketError::InvalidOracle)));
    }

    #[tokio::test]
    async fn test_resolve_already_resolved() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Set market to expired and set oracle ID to match caller
        market_contract.market.status = MarketStatus::Expired;
        let caller = market_contract.get_caller_address().await.unwrap();
        market_contract.market.oracle_id = format!("{:?}", caller);

        // Submit outcome to oracle
        let oracle_manager = market_contract.get_oracle_manager().await.unwrap();
        oracle_manager.submit_outcome(market_contract.market.yes_token_address.clone(), true).await.unwrap();

        // Resolve market first time
        market_contract.resolve().await.unwrap();

        // Try to resolve again
        let result = market_contract.resolve().await;
        assert!(matches!(result, Err(MarketError::MarketAlreadyResolved)));
    }

    #[tokio::test]
    async fn test_cancel_order() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Mint some tokens and place an order
        market_contract.mint_tokens(U256::from(100)).await.unwrap();
        market_contract.place_order(Side::Buy, U256::from(50), U256::from(10)).await.unwrap();

        // Cancel the order
        let result = market_contract.cancel_order("test_order_id".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_all_user_orders() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Mint some tokens and place an order
        market_contract.mint_tokens(U256::from(100)).await.unwrap();
        market_contract.place_order(Side::Buy, U256::from(50), U256::from(10)).await.unwrap();

        // Cancel all user orders
        let result = market_contract.cancel_all_user_orders().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_expiry() {
        let (mut market_contract, _) = setup_test_market().await;
        
        // Set market to expire in 1 second
        market_contract.market.expiry_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 1;

        // Wait for 3 seconds to allow background task to run
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // Check expiry
        assert_eq!(market_contract.market.status, MarketStatus::Expired);
    }
} 
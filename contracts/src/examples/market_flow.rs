use crate::{
    market::{MarketContractState, MarketError},
    market_factory::{MarketFactoryState, MarketFactoryError},
    auth::AuthManager,
    events::EventLogger,
};
use ethers::types::{Address, U256};
use hyperliquid_rust_sdk::types::Side;
use std::sync::Arc;

/// This example demonstrates a complete flow of:
/// 1. Creating a market
/// 2. Users depositing collateral
/// 3. Minting and trading tokens
/// 4. Market resolution and settlement
async fn run_market_example() -> Result<(), Box<dyn std::error::Error>> {
    // Setup components
    let auth_manager = Arc::new(AuthManager::new("http://localhost:8545").await?);
    let event_logger = Arc::new(EventLogger::new(true, false, None));

    // Create market factory
    let mut factory = MarketFactoryState::new(
        "http://localhost:8080",
        auth_manager.clone(),
    ).await?;

    // Create a new market
    let market_id = factory.create_market(
        "Will ETH be above $4000 on Dec 31, 2024?".to_string(),
        1735689600, // Dec 31, 2024
        Address::zero(), // Oracle address
        "USDC".to_string(),
    ).await?;

    println!("Created market: {}", market_id);

    // Get the market instance
    let market = factory.get_market(market_id.clone())
        .ok_or("Market not found")?;

    // Create market contract instance
    let mut market_contract = MarketContractState::new(
        market,
        "http://localhost:8080",
        auth_manager.clone(),
        event_logger.clone(),
    ).await?;

    // User 1: Deposit collateral and mint tokens
    println!("User 1: Depositing collateral...");
    market_contract.deposit_collateral(U256::from(1000)).await?;

    println!("User 1: Minting tokens...");
    market_contract.mint_tokens(U256::from(100)).await?;

    // User 1: Place a sell order for YES tokens
    println!("User 1: Placing sell order...");
    market_contract.place_order(
        Side::Sell,
        U256::from(60), // Sell at $0.60
        U256::from(50), // Sell 50 tokens
    ).await?;

    // User 2: (In reality would be a different wallet)
    // Deposit collateral and buy YES tokens
    println!("User 2: Depositing collateral...");
    market_contract.deposit_collateral(U256::from(1000)).await?;

    println!("User 2: Placing buy order...");
    market_contract.place_order(
        Side::Buy,
        U256::from(60), // Buy at $0.60
        U256::from(50), // Buy 50 tokens
    ).await?;

    // Time passes... market expires
    println!("Market expiring...");
    market_contract.market.status = crate::MarketStatus::Expired;

    // Oracle resolves the market
    println!("Oracle resolving market...");
    market_contract.resolve(true).await?; // YES wins

    // Users claim winnings
    println!("Users claiming winnings...");
    let winnings = market_contract.claim_winnings().await?;
    println!("Claimed winnings: {} USDC", winnings);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_flow() {
        let result = run_market_example().await;
        assert!(result.is_ok());
    }
} 
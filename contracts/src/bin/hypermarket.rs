use clap::{Parser, Subcommand};
use hypermarket::{
    market::{MarketContractState, MarketError},
    market_factory::{MarketFactoryState, MarketFactoryError},
    auth::AuthManager,
    events::EventLogger,
};
use ethers::types::{Address, U256};
use hyperliquid_rust::types::order::Side;
use std::sync::Arc;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, env = "HYPERLIQUID_API_URL")]
    api_url: String,

    #[arg(long, env = "HYPERLIQUID_PRIVATE_KEY")]
    private_key: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new market
    CreateMarket {
        /// Market question
        question: String,
        /// Expiry timestamp
        #[arg(long)]
        expiry: u64,
        /// Oracle address
        #[arg(long)]
        oracle: String,
        /// Collateral token
        #[arg(long, default_value = "USDC")]
        collateral: String,
    },
    /// List all markets
    ListMarkets,
    /// Deposit collateral
    DepositCollateral {
        /// Market ID
        #[arg(long)]
        market_id: String,
        /// Amount to deposit
        amount: u64,
    },
    /// Mint tokens
    MintTokens {
        /// Market ID
        #[arg(long)]
        market_id: String,
        /// Amount to mint
        amount: u64,
    },
    /// Place an order
    PlaceOrder {
        /// Market ID
        #[arg(long)]
        market_id: String,
        /// Buy or Sell
        #[arg(long)]
        side: String,
        /// Price in cents
        #[arg(long)]
        price: u64,
        /// Amount of tokens
        amount: u64,
    },
    /// Claim winnings
    ClaimWinnings {
        /// Market ID
        #[arg(long)]
        market_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Setup components
    let auth_manager = Arc::new(AuthManager::new(&cli.api_url).await?);
    auth_manager.connect_wallet(&cli.private_key).await?;
    let event_logger = Arc::new(EventLogger::new(true, false, None));

    // Create market factory
    let mut factory = MarketFactoryState::new(
        &cli.api_url,
        auth_manager.clone(),
    ).await?;

    match cli.command {
        Commands::CreateMarket { question, expiry, oracle, collateral } => {
            let oracle_addr = oracle.parse::<Address>()?;
            let market_id = factory.create_market(
                question,
                expiry,
                oracle_addr,
                collateral,
            ).await?;
            println!("Created market: {}", market_id);
        }

        Commands::ListMarkets => {
            let markets = factory.list_markets();
            for (id, market) in markets {
                println!("Market {}: {}", id, market.question);
                println!("  Status: {:?}", market.status);
                println!("  Expiry: {}", market.expiry_timestamp);
                println!("  Oracle: {}", market.oracle_id);
                println!();
            }
        }

        Commands::DepositCollateral { market_id, amount } => {
            let market = factory.get_market(market_id.clone())
                .ok_or("Market not found")?;
            
            let mut market_contract = MarketContractState::new(
                market,
                &cli.api_url,
                auth_manager.clone(),
                event_logger.clone(),
            ).await?;

            market_contract.deposit_collateral(U256::from(amount)).await?;
            println!("Deposited {} USDC as collateral", amount);
        }

        Commands::MintTokens { market_id, amount } => {
            let market = factory.get_market(market_id.clone())
                .ok_or("Market not found")?;
            
            let mut market_contract = MarketContractState::new(
                market,
                &cli.api_url,
                auth_manager.clone(),
                event_logger.clone(),
            ).await?;

            market_contract.mint_tokens(U256::from(amount)).await?;
            println!("Minted {} YES/NO tokens", amount);
        }

        Commands::PlaceOrder { market_id, side, price, amount } => {
            let market = factory.get_market(market_id.clone())
                .ok_or("Market not found")?;
            
            let mut market_contract = MarketContractState::new(
                market,
                &cli.api_url,
                auth_manager.clone(),
                event_logger.clone(),
            ).await?;

            let side = match side.to_lowercase().as_str() {
                "buy" => Side::Buy,
                "sell" => Side::Sell,
                _ => return Err("Invalid side. Use 'buy' or 'sell'".into()),
            };

            market_contract.place_order(
                side,
                U256::from(price),
                U256::from(amount),
            ).await?;
            println!("Placed order: {} {} tokens at ${}.{:02}", 
                side, amount, price / 100, price % 100);
        }

        Commands::ClaimWinnings { market_id } => {
            let market = factory.get_market(market_id.clone())
                .ok_or("Market not found")?;
            
            let mut market_contract = MarketContractState::new(
                market,
                &cli.api_url,
                auth_manager.clone(),
                event_logger.clone(),
            ).await?;

            let winnings = market_contract.claim_winnings().await?;
            println!("Claimed {} USDC in winnings", winnings);
        }
    }

    Ok(())
} 
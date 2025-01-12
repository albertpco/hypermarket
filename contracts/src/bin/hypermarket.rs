use clap::{Parser, Subcommand};
use hypermarket::{
    market::{MarketContractState},
    market_factory::{MarketFactoryState},
    auth::AuthManager,
    events::EventLogger,
    MarketContract,
    MarketFactory,
};
use ethers::types::{Address, U256};
use std::sync::Arc;
use std::str::FromStr;

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
    // Initialize logging
    env_logger::init();

    // Load .env file if it exists
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    // Log configuration (but mask private key)
    log::info!("Using API URL: {}", cli.api_url);
    log::info!("Private key is set: {}", !cli.private_key.is_empty());

    // Setup components
    let auth_manager = Arc::new(AuthManager::new(&cli.api_url).await?);
    auth_manager.connect_wallet(&cli.private_key).await?;
    let event_logger = Arc::new(EventLogger::new(true, false, None));

    // Create market factory
    let mut factory = MarketFactoryState::new(
        &cli.api_url,
        auth_manager.clone(),
        event_logger.clone(),
        U256::from(100),
    ).await?;

    match cli.command {
        Commands::CreateMarket { question, expiry, oracle, collateral } => {
            let oracle_addr = Address::from_str(&oracle)?;
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
                auth_manager.clone(),
                event_logger.clone(),
                Default::default(),
            );

            market_contract.mint_tokens(amount).await?;
            println!("Deposited {} USDC as collateral", amount);
        }

        Commands::MintTokens { market_id, amount } => {
            let market = factory.get_market(market_id.clone())
                .ok_or("Market not found")?;
            
            let mut market_contract = MarketContractState::new(
                market,
                auth_manager.clone(),
                event_logger.clone(),
                Default::default(),
            );

            market_contract.mint_tokens(amount).await?;
            println!("Minted {} YES/NO tokens", amount);
        }

        Commands::PlaceOrder { market_id, side: _, price: _, amount: _ } => {
            let market = factory.get_market(market_id.clone())
                .ok_or("Market not found")?;
            
            println!("Order placement not yet implemented for market: {}", market.question);
        }

        Commands::ClaimWinnings { market_id } => {
            let market = factory.get_market(market_id.clone())
                .ok_or("Market not found")?;
            
            let mut market_contract = MarketContractState::new(
                market,
                auth_manager.clone(),
                event_logger.clone(),
                Default::default(),
            );

            let winnings = market_contract.claim_winnings().await?;
            println!("Claimed {} USDC in winnings", winnings);
        }
    }

    Ok(())
} 
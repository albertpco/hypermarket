use ethers::types::{Address, H256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEvent {
    MarketCreated {
        market_id: String,
        creator: Address,
        question: String,
        expiry_timestamp: u64,
        oracle_id: Address,
        yes_token: String,
        no_token: String,
        timestamp: u64,
    },
    TokensMinted {
        market_id: String,
        user: Address,
        amount: u64,
        timestamp: u64,
    },
    TokensBurned {
        market_id: String,
        user: Address,
        yes_amount: u64,
        no_amount: u64,
        timestamp: u64,
        tx_hash: H256,
    },
    MarketResolved {
        market_id: String,
        oracle: Address,
        outcome: bool,
        timestamp: u64,
        tx_hash: H256,
    },
    WinningsClaimed {
        market_id: String,
        user: Address,
        amount: u64,
        timestamp: u64,
        tx_hash: H256,
    },
    OrderPlaced {
        market_id: String,
        user: Address,
        side: String,
        price: u64,
        amount: u64,
        timestamp: u64,
        tx_hash: H256,
    },
    OrderCancelled {
        market_id: String,
        user: Address,
        order_id: String,
        timestamp: u64,
        tx_hash: H256,
    },
    OrdersCancelled {
        market_id: String,
        user: Address,
        timestamp: u64,
    },
    MarketExpired {
        market_id: String,
        timestamp: u64,
    },
    CollateralDeposited {
        market_id: String,
        user: Address,
        amount: u64,
        timestamp: u64,
        tx_hash: H256,
    },
    CollateralWithdrawn {
        market_id: String,
        user: Address,
        amount: u64,
        timestamp: u64,
        tx_hash: H256,
    },
    OracleAdded {
        oracle_id: Address,
        timestamp: u64,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OracleEvent {
    OutcomeSubmitted {
        market_id: String,
        oracle: Address,
        outcome: bool,
        timestamp: u64,
    },
}

#[async_trait::async_trait]
pub trait EventEmitter: Send + Sync + std::fmt::Debug {
    fn emit_market_event(&self, event: MarketEvent);
    fn emit_oracle_event(&self, event: OracleEvent);
}

#[derive(Debug)]
pub struct EventLogger {
    pub log_to_console: bool,
    pub log_to_file: bool,
    pub log_file_path: Option<String>,
}

impl EventLogger {
    pub fn new(log_to_console: bool, log_to_file: bool, log_file_path: Option<String>) -> Self {
        Self {
            log_to_console,
            log_to_file,
            log_file_path,
        }
    }
}

impl EventEmitter for EventLogger {
    fn emit_market_event(&self, event: MarketEvent) {
        if self.log_to_console {
            println!("Market Event: {:?}", event);
        }

        if self.log_to_file {
            if let Some(path) = &self.log_file_path {
                // Implement file logging here
                // For now, we'll just print that we would log to file
                println!("Would log market event to file: {}", path);
            }
        }
    }

    fn emit_oracle_event(&self, event: OracleEvent) {
        if self.log_to_console {
            println!("Oracle Event: {:?}", event);
        }

        if self.log_to_file {
            if let Some(path) = &self.log_file_path {
                // Implement file logging here
                // For now, we'll just print that we would log to file
                println!("Would log oracle event to file: {}", path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_logger() {
        let logger = EventLogger::new(true, false, None);

        // Test market event
        let market_event = MarketEvent::TokensMinted {
            market_id: "test_market".to_string(),
            user: Address::zero(),
            amount: 100,
            timestamp: 1234567890,
        };
        logger.emit_market_event(market_event);

        // Test oracle event
        let oracle_event = OracleEvent::OutcomeSubmitted {
            market_id: "test_market".to_string(),
            oracle: Address::zero(),
            outcome: true,
            timestamp: 1234567890,
        };
        logger.emit_oracle_event(oracle_event);
    }
} 
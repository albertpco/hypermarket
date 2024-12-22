use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

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
        amount: U256,
        timestamp: u64,
    },
    OrderPlaced {
        market_id: String,
        user: Address,
        side: String,
        price: U256,
        amount: U256,
        timestamp: u64,
    },
    MarketResolved {
        market_id: String,
        oracle: Address,
        outcome: bool,
        timestamp: u64,
    },
    WinningsClaimed {
        market_id: String,
        user: Address,
        amount: U256,
        timestamp: u64,
    },
    TokensBurned {
        market_id: String,
        user: Address,
        yes_amount: U256,
        no_amount: U256,
        timestamp: u64,
    },
    CollateralDeposited {
        market_id: String,
        user: Address,
        amount: U256,
        timestamp: u64,
    },
    CollateralWithdrawn {
        market_id: String,
        user: Address,
        amount: U256,
        timestamp: u64,
    },
    CollateralRequirementUpdated {
        market_id: String,
        user: Address,
        required_amount: U256,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OracleEvent {
    OracleRegistered {
        oracle_id: Address,
        registrar: Address,
        timestamp: u64,
    },
    OutcomeSubmitted {
        market_id: String,
        oracle_id: Address,
        outcome: bool,
        timestamp: u64,
    },
    ReputationUpdated {
        oracle_id: Address,
        old_score: u32,
        new_score: u32,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthEvent {
    UserAuthenticated {
        user: Address,
        timestamp: u64,
    },
    SignatureCreated {
        user: Address,
        action: String,
        timestamp: u64,
    },
}

pub trait EventEmitter {
    fn emit_market_event(&self, event: MarketEvent);
    fn emit_oracle_event(&self, event: OracleEvent);
    fn emit_auth_event(&self, event: AuthEvent);
}

#[derive(Clone)]
pub struct EventLogger {
    pub enable_console: bool,
    pub enable_file: bool,
    log_file: Option<String>,
}

impl EventLogger {
    pub fn new(enable_console: bool, enable_file: bool, log_file: Option<String>) -> Self {
        Self {
            enable_console,
            enable_file,
            log_file,
        }
    }

    fn get_current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn log_event<T: Serialize>(&self, event: &T, event_type: &str) {
        let timestamp = Self::get_current_timestamp();
        let event_json = serde_json::to_string_pretty(event).unwrap_or_default();

        if self.enable_console {
            println!("[{}] {} Event: {}", timestamp, event_type, event_json);
        }

        if self.enable_file {
            if let Some(file_path) = &self.log_file {
                use std::fs::OpenOptions;
                use std::io::Write;

                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path)
                {
                    let _ = writeln!(
                        file,
                        "[{}] {} Event: {}",
                        timestamp, event_type, event_json
                    );
                }
            }
        }
    }
}

impl EventEmitter for EventLogger {
    fn emit_market_event(&self, event: MarketEvent) {
        self.log_event(&event, "Market");
    }

    fn emit_oracle_event(&self, event: OracleEvent) {
        self.log_event(&event, "Oracle");
    }

    fn emit_auth_event(&self, event: AuthEvent) {
        self.log_event(&event, "Auth");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_console_logging() {
        let logger = EventLogger::new(true, false, None);
        
        let event = MarketEvent::MarketCreated {
            market_id: "TEST_1".to_string(),
            creator: Address::zero(),
            question: "Test Market?".to_string(),
            expiry_timestamp: 1234567890,
            oracle_id: Address::zero(),
            yes_token: "YES".to_string(),
            no_token: "NO".to_string(),
            timestamp: EventLogger::get_current_timestamp(),
        };

        logger.emit_market_event(event);
    }

    #[test]
    fn test_file_logging() {
        let temp_file = NamedTempFile::new().unwrap();
        let logger = EventLogger::new(
            false,
            true,
            Some(temp_file.path().to_str().unwrap().to_string()),
        );

        let event = OracleEvent::OracleRegistered {
            oracle_id: Address::zero(),
            registrar: Address::zero(),
            timestamp: EventLogger::get_current_timestamp(),
        };

        logger.emit_oracle_event(event);

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("Oracle Event"));
    }
} 
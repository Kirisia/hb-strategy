use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SendParcel {
    ConfigError(String),
    StrategyState(String),
}

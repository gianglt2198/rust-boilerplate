use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TargetLoggingConfig {
    pub target: Option<String>,
    pub path: Option<String>,
    pub level: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub targets: Option<Vec<TargetLoggingConfig>>,
}

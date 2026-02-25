use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NatsConfig {
    pub enabled: bool,
    pub url: Option<String>,
}

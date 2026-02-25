use serde::{Deserialize, Serialize};

use ro_config::config::SharedConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub shutdown_timeout: u64,
    pub cors: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    #[serde(flatten)]
    pub shared: SharedConfig,
    pub server: ServerConfig,
}

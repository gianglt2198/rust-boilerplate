pub mod db;
pub mod log;
pub mod nats;
pub mod otel;

use serde::{Deserialize, Serialize};

use crate::config::{db::DatabaseConfig, log::LoggingConfig, nats::NatsConfig, otel::OtelConfig};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommonConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub debug: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SharedConfig {
    pub common: CommonConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub nats: NatsConfig,
    pub otel: OtelConfig,
}

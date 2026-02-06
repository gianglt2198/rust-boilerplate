use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommonConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub debug: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub shutdown_timeout: u64,
    pub cors: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExporterOtelConfig {
    pub protocol: String,
    pub endpoint: String,
    pub timeout: u16,
    pub batch_size: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OtelConfig {
    pub enabled: bool,
    pub exporter: ExporterOtelConfig,
}

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NatsConfig {
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub common: CommonConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub otel: OtelConfig,
    pub logging: LoggingConfig,
    pub nats: NatsConfig,
}

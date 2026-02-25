use serde::{Deserialize, Serialize};

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

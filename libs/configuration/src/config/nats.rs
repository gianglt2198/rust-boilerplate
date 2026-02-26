use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NatsConfig {
    pub enabled: bool,
    pub url: String,
    /// Subject prefix prepended to every topic.
    /// e.g. `base_path = "myapp"` → `"user.created"` → `"myapp.user.created"`
    /// Leave empty to disable prefixing.
    pub base_path: String,

    /// How often to ping the NATS server to check liveness (seconds).
    #[serde(default = "NatsConfig::default_ping_interval")]
    pub ping_interval_secs: u64,

    /// Whether to auto-reconnect on disconnect.
    #[serde(default = "NatsConfig::default_allow_reconnect")]
    pub allow_reconnect: bool,

    /// Maximum reconnect attempts (-1 = infinite).
    #[serde(default = "NatsConfig::default_max_reconnects")]
    pub max_reconnects: i32,
}
impl NatsConfig {
    fn default_ping_interval() -> u64 {
        20
    }
    fn default_allow_reconnect() -> bool {
        true
    }
    fn default_max_reconnects() -> i32 {
        -1
    }

    pub fn ping_interval(&self) -> Duration {
        Duration::from_secs(self.ping_interval_secs)
    }
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            url: "nats://localhost:4222".to_string(),
            base_path: String::new(),
            ping_interval_secs: Self::default_ping_interval(),
            allow_reconnect: Self::default_allow_reconnect(),
            max_reconnects: Self::default_max_reconnects(),
        }
    }
}

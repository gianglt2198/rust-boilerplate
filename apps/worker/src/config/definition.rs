use serde::{Deserialize, Serialize};

use ro_config::config::SharedConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkerConfig {
    #[serde(flatten)]
    pub shared: SharedConfig,
}

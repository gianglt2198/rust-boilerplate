use std::{env, sync::OnceLock};

use crate::config::definition::WorkerConfig;

use ro_config as config;

impl WorkerConfig {
    pub fn get_config() -> &'static WorkerConfig {
        static CONFIG: OnceLock<WorkerConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            let newconfig = config::loader::Loader::new(Some("APP"))
                .load_yaml(
                    env::var("APP_FILE_PATH").unwrap_or_else(|_e| "./config.yaml".to_string()),
                )
                .expect("can't load from config")
                .load_dotenv()
                .expect("can't load from env");

            let cfg: WorkerConfig = newconfig.deserialize().expect("can't parse config");
            cfg
        })
    }
}

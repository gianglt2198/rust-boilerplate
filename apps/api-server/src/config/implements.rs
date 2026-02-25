use std::{env, sync::OnceLock};

use crate::config::definition::AppConfig;

use ro_config as config;

impl AppConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    pub fn get_config() -> &'static AppConfig {
        static CONFIG: OnceLock<AppConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            let newconfig = config::loader::Loader::new(Some("APP"))
                .load_yaml(
                    env::var("APP_FILE_PATH").unwrap_or_else(|_e| "./config.yaml".to_string()),
                )
                .expect("can't load from config")
                .load_dotenv()
                .expect("can't load from env");

            let cfg: AppConfig = newconfig.deserialize().expect("can't parse config");
            cfg
        })
    }
}

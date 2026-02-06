use std::{env, sync::OnceLock};

use crate::{definition::AppConfig, loader::Loader};

impl AppConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    pub fn nats_addr(&self) -> String {
        self.nats
            .url
            .clone()
            .unwrap_or("nats://localhost:4222".to_string())
    }

    pub fn get_config() -> &'static AppConfig {
        static CONFIG: OnceLock<AppConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            let newconfig = Loader::new(Some("APP"))
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

    pub fn get_db_addr(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database,
        )
    }
}

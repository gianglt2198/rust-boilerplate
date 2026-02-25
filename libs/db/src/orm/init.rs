use std::{sync::Arc, time::Duration};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use ro_config::config::db::DatabaseConfig;

pub async fn new_db(db_cfg: DatabaseConfig) -> Result<Arc<DatabaseConnection>, anyhow::Error> {
    let mut opts = ConnectOptions::new(db_cfg.get_addr());
    opts.max_connections(db_cfg.pool_size)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true); // Auto-logs SQL queries to tracing!

    let db = Database::connect(opts).await?;
    Ok(Arc::new(db))
}

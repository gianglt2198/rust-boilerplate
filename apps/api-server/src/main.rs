mod config;
mod middlewares;
mod routes;
mod states;

use anyhow::Result;
use opentelemetry::trace::TracerProvider;
use ro_adapters::database::postgres::user_repo::PUserRepository;
use ro_core::services::user_service::UserService;
use ro_messaging::nats::{NatsClient, middleware::tracing_middleware as nats_tracing_mw};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{
    fmt::writer::MakeWriterExt, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::config::definition::AppConfig;
use crate::middlewares::request_id;

use ro_db::orm;
use ro_telemetry::{
    meter::{self, collect_system_metrics},
    tracer,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg: &AppConfig = AppConfig::get_config();

    let trace_provider = tracer::init_tracer(
        cfg.shared.common.name.clone(),
        cfg.shared.otel.exporter.endpoint.clone(),
    );
    let trace = trace_provider.tracer(cfg.shared.common.name.clone());

    let _ = meter::init_meter(
        cfg.shared.common.name.clone(),
        cfg.shared.otel.exporter.endpoint.clone(),
    );

    let logfile = tracing_appender::rolling::hourly("./logs", "rolling.log");
    let stdout = std::io::stdout.with_max_level(tracing::Level::INFO);

    tracing_subscriber::registry()
        //     .with(
        //         tracing_subscriber::EnvFilter::try_from_default_env()
        //             .unwrap_or_else(|_| cfg.logging.level.clone().into()),
        //     )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(stdout.and(logfile))
                .json(),
        )
        .with(tracing_opentelemetry::layer().with_tracer(trace))
        .init();

    tracing::info!("Starting Rust Observability");
    tracing::info!("Configuration: {:?}", cfg);

    let db = orm::new_db(cfg.shared.database.clone()).await?;
    // 1. Create Adapter (Repository)
    let user_repo = PUserRepository::new(Arc::clone(&db));

    let nats = NatsClient::connect(
        cfg.shared.common.name.clone(),
        cfg.shared.nats.clone(),
        vec![nats_tracing_mw()],
    )
    .await?;

    // 2. Create Service (Inject Repository)
    let user_service = UserService::new(Arc::new(user_repo), Arc::new(nats));

    // 3. Create State (Inject Service)
    let state = Arc::new(states::AppState::new(user_service));

    let cors: CorsLayer = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let app = routes::create_router(state)
        .layer(axum::middleware::from_fn(
            middlewares::metrics::metric_middleware,
        ))
        .layer(axum::middleware::from_fn(
            middlewares::tracing::tracing_middleware,
        ))
        .layer(axum::middleware::from_fn(request_id::request_id_middleware))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&cfg.addr()).await?;
    tracing::info!("Server listening on {}", cfg.addr());

    tokio::spawn(async move {
        collect_system_metrics(5).await;
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shut down gracefully");

    Ok(())
}

/// Handle graceful shutdown Ctrl+C
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler")
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down...");
        },
    };
}

mod config;

use ro_core::domain::entities::user::User;
use ro_messaging::{
    QueueSubscriber, handler,
    nats::{NatsClient, middleware::tracing_middleware},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::definition::WorkerConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load Config & Telemetry
    let cfg = WorkerConfig::get_config();
    // ro_telemetry::init_subscriber(...) // Initialize logging

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Worker starting...");

    let nats = NatsClient::connect(
        cfg.shared.common.name.clone(),
        cfg.shared.nats.clone(),
        vec![tracing_middleware()],
    )
    .await?;

    nats.queue_subscribe(
        "user.created",
        "worker-group",
        handler(|msg| async move {
            let user = msg.json::<User>()?;
            tracing::info!(username = %user.username, id = %user.id, "user.created received");
            // inject services here
            Ok(())
        }),
    )
    .await?;

    // // 2. Connect to NATS
    // // (Assumes you add nats_url to your config, hardcoded for demo)
    // let client = ConnectOptions::new().connect(cfg.nats_addr()).await?;

    // // 3. Subscribe
    // let mut subscriber = client.subscribe("user.created").await?;
    // tracing::info!("Listening on 'user.created'...");

    // // 4. Process Loop
    // while let Some(message) = subscriber.next().await {
    //     if let Ok(user) = serde_json::from_slice::<User>(&message.payload) {
    //         tracing::info!("WORKER RECEIVED: User created -> {}", user.username);

    //         // HERE IS THE POWER:
    //         // You can now inject the SAME Service used in the API
    //         // to do other business logic (e.g., send welcome email).
    //         // let email_service = EmailService::new(...);
    //         // email_service.send_welcome(user).await;
    //     }
    // }
    tokio::signal::ctrl_c().await?;
    nats.close().await?;
    Ok(())
}

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::states::SharedState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[tracing::instrument(name = "liveness")]
pub async fn liveness() -> StatusCode {
    StatusCode::OK
}

#[tracing::instrument(name = "readiness")]
pub async fn readiness(State(_state): State<SharedState>) -> Json<HealthResponse> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Json(response)
}

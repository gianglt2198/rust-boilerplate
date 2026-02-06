use axum::{
    Router,
    routing::{get, post},
};

use crate::states::SharedState;

pub mod health;
pub mod users;

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/health/liveness", get(health::liveness))
        .route("/health/readiness", get(health::readiness))
        // User Routes
        .route("/users", post(users::create_user))
        .route("/users/:id", get(users::get_user))
        .with_state(state)
}

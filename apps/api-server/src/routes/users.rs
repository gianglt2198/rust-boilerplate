use crate::states::SharedState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

// DTO: Request Body
#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

// DTO: Response Body
#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
}

// Handler: Create User
pub async fn create_user(
    State(state): State<SharedState>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    match state
        .user_service
        .register_user(payload.username, payload.email)
        .await
    {
        Ok(user) => {
            let response = UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            // In a real app, map Domain Errors to HTTP Status Codes properly
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

// Handler: Get User
pub async fn get_user(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.user_service.get_user(&id).await {
        Ok(user) => {
            let response = UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
            };
            Json(response).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "User not found").into_response(),
    }
}

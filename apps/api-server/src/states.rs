use std::sync::Arc;

use ro_core::services::user_service::UserService;

#[derive(Debug, Clone)]
pub struct AppState {
    pub user_service: UserService,
}

impl AppState {
    pub fn new(user_service: UserService) -> Self {
        Self { user_service }
    }
}

pub type SharedState = Arc<AppState>;

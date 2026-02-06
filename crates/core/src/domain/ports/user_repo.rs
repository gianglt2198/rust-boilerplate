use async_trait::async_trait;
use core::fmt::Debug;
use thiserror::Error;

use crate::domain::entities::user::User;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    #[error("Database error: {0}")]
    System(String),
}

#[async_trait]
pub trait UserRepository: Send + Sync + 'static + Debug {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, UserError>;
    async fn save(&self, user: &User) -> Result<(), UserError>;
}

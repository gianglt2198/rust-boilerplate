use crate::domain::entities::user::User;
use async_trait::async_trait;
use core::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessageError {
    #[error("Failed to publish: {0}")]
    PublishError(String),
}

#[async_trait]
pub trait EventPublisher: Send + Sync + Debug {
    async fn publish_user_created(&self, user: &User) -> Result<(), MessageError>;
}

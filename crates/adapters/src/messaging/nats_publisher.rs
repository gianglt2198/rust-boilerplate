use async_nats::Client;
use async_trait::async_trait;
use ro_core::domain::{
    entities::user::User,
    ports::messaging::{EventPublisher, MessageError},
};

#[derive(Debug)]
pub struct NatsPublisher {
    client: Client,
}

impl NatsPublisher {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl EventPublisher for NatsPublisher {
    async fn publish_user_created(&self, user: &User) -> Result<(), MessageError> {
        let payload =
            serde_json::to_vec(user).map_err(|e| MessageError::PublishError(e.to_string()))?;

        self.client
            .publish("user.created", payload.into())
            .await
            .map_err(|e| MessageError::PublishError(e.to_string()))?;

        Ok(())
    }
}

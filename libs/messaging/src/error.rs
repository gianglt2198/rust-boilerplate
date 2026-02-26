use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessagingError {
    /// Failure during message publish
    #[error("Publish failed: {0}")]
    Publish(String),

    /// Failure creating/maintaining a subscriber
    #[error("Subscribe failed: {0}")]
    Subscribe(String),

    /// Failure removing a subscription
    #[error("Unsubscribe failed: {0}")]
    Unsubscribe(String),

    /// Request/reply pattern failure (includes timeout)
    #[error("Request failed: {0}")]
    Request(String),

    /// Could not serialize outbound payload
    #[error("Serialization failed: {0}")]
    Serialization(String),

    /// Could not deserialize inbound payload
    #[error("Deserialization failed: {0}")]
    Deserialization(String),

    /// Handler returned an error
    #[error("Handler error: {0}")]
    Handler(String),

    /// Connection has been closed / drained
    #[error("Connection closed")]
    Closed,
}

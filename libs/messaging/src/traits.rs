use std::{collections::HashMap, pin::Pin, sync::Arc};

use async_trait::async_trait;
use bytes::Bytes;
use std::time::Duration;

use crate::{Message, MessagingError};

pub type HandlerFuture =
    Pin<Box<dyn Future<Output = Result<Option<Bytes>, MessagingError>> + Send>>;

pub type Handler = Arc<dyn Fn(Message) -> HandlerFuture + Send + Sync>;

pub fn handler<F, Fut>(f: F) -> Handler
where
    F: Fn(Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), MessagingError>> + Send + 'static,
{
    Arc::new(move |msg| {
        let fut = f(msg);
        Box::pin(async move { fut.await.map(|_| None) })
    })
}

pub fn reply_handler<F, Fut>(f: F) -> Handler
where
    F: Fn(Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Option<Bytes>, MessagingError>> + Send + 'static,
{
    Arc::new(move |msg| Box::pin(f(msg)))
}

/// Publish raw or structured messages to a topic.
#[async_trait]
pub trait Publisher: Send + Sync {
    async fn publish(
        &self,
        topic: &str,
        data: Bytes,
        attrs: HashMap<String, String>,
    ) -> Result<(), MessagingError>;

    async fn publish_json<T: serde::Serialize + Send + Sync>(
        &self,
        topic: &str,
        payload: &T,
    ) -> Result<(), MessagingError> {
        let data = serde_json::to_vec(payload)
            .map_err(|e| MessagingError::Serialization(e.to_string()))?;
        self.publish(topic, Bytes::from(data), HashMap::new()).await
    }

    async fn close(&self) -> Result<(), MessagingError>;
}

/// Fan-out pub/sub subscriber.
#[async_trait]
pub trait Subscriber: Send + Sync {
    async fn subscribe(&self, topic: &str, handler: Handler) -> Result<(), MessagingError>;

    async fn unsubscribe(&self, topic: &str) -> Result<(), MessagingError>;
    async fn close(&self) -> Result<(), MessagingError>;
}

/// Competing-consumer (work-queue) subscriber.
///
/// Only one subscriber in the same `group` will receive each message.
#[async_trait]
pub trait QueueSubscriber: Send + Sync {
    async fn queue_subscribe(
        &self,
        topic: &str,
        group: &str,
        handler: Handler,
    ) -> Result<(), MessagingError>;

    async fn unsubscribe(&self, topic: &str) -> Result<(), MessagingError>;

    async fn close(&self) -> Result<(), MessagingError>;
}

/// Request / reply broker.
#[async_trait]
pub trait Broker: Send + Sync {
    /// Send a request to `pattern`, wait up to `timeout`, deserialize reply as `R`.
    async fn request<T, R>(
        &self,
        pattern: &str,
        payload: &T,
        attrs: HashMap<String, String>,
        timeout: Duration,
    ) -> Result<R, MessagingError>
    where
        T: serde::Serialize + Send + Sync,
        R: serde::de::DeserializeOwned;

    async fn close(&self) -> Result<(), MessagingError>;
}

/// `Publisher + Subscriber` — the standard fan-out pubsub client.
pub trait Client: Publisher + Subscriber {}
impl<T: Publisher + Subscriber> Client for T {}

/// `Publisher + QueueSubscriber` — the work-queue client.
pub trait QueueClient: Publisher + QueueSubscriber {}
impl<T: Publisher + QueueSubscriber> QueueClient for T {}

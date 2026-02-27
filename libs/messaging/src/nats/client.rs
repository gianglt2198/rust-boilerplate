use std::{collections::HashMap, sync::Arc, time::Duration};

use async_nats::ConnectOptions;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;
use tokio::sync::Mutex;

use ro_config::config::nats::NatsConfig;

use crate::{
    Broker, Handler, MessagingError, Publisher, QueueSubscriber, Subscriber,
    nats::{
        factory::MessageFactory,
        middleware::{MiddlewareFn, NatsHandlerFn, apply_middleware},
    },
};

#[derive(Debug, Clone)]
pub struct NatsClient {
    inner: async_nats::Client,
    factory: Arc<MessageFactory>,
    middlewares: Arc<Vec<MiddlewareFn>>,
    /// topic → abort handle for the subscription drain task
    subscriptions: Arc<Mutex<HashMap<String, tokio::task::AbortHandle>>>,
}

impl NatsClient {
    pub async fn connect(
        name: String,
        cfg: NatsConfig,
        middlewares: Vec<MiddlewareFn>,
    ) -> Result<Self, MessagingError> {
        if !cfg.enabled {
            return Err(MessagingError::Closed);
        }

        let url = cfg.url.clone();
        let ping = cfg.ping_interval();
        let allow_reconnect = cfg.allow_reconnect;
        let max_reconnects = cfg.max_reconnects;
        let name = name.clone();

        let cfg = Arc::new(cfg);
        let factory = Arc::new(MessageFactory::new(name.clone(), Arc::clone(&cfg)));

        let mut opts = ConnectOptions::new().name(&name).ping_interval(ping);

        if !allow_reconnect {
            opts = opts.max_reconnects(Some(0));
        } else if max_reconnects >= 0 {
            opts = opts.max_reconnects(Some(max_reconnects as usize));
        }
        // max_reconnects < 0 → infinite (default in async-nats)

        opts = opts
            .reconnect_delay_callback(|attempts| {
                tracing::info!("NATS: reconnected");
                std::time::Duration::from_millis(std::cmp::min((attempts * 100) as u64, 8000))
            })
            .event_callback(|event| async move {
                match event {
                    async_nats::Event::Disconnected => println!("disconnected"),
                    async_nats::Event::Connected => println!("reconnected"),
                    async_nats::Event::ClientError(err) => {
                        println!("client error occurred: {}", err)
                    }
                    other => println!("other event happened: {}", other),
                }
            });

        let inner = opts
            .connect(url)
            .await
            .map_err(|e| MessagingError::Subscribe(e.to_string()))?;

        tracing::info!(name, "NATS: connected");

        Ok(Self {
            inner,
            factory,
            middlewares: Arc::new(middlewares),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Access the raw `async_nats::Client` for advanced use cases.
    pub fn inner(&self) -> &async_nats::Client {
        &self.inner
    }

    /// Wrap a domain `Handler` into a transport-level `NatsHandlerFn`
    /// (factory reads the message and converts it, then calls the domain handler).
    fn wrap_handler(&self, handler: Handler) -> NatsHandlerFn {
        let factory = Arc::clone(&self.factory);
        Arc::new(move |nats_msg: async_nats::Message| {
            let handler = Arc::clone(&handler);
            let factory = Arc::clone(&factory);
            Box::pin(async move {
                let msg = factory.read_message(nats_msg)?;
                handler(msg)
                    .await
                    .map(|_| ())
                    .map_err(|e| MessagingError::Handler(e.to_string()))
            })
        })
    }

    /// Spawn a drain loop for a subscription stream; return the `AbortHandle`.
    ///
    /// One Tokio task per subscription (same as Go goroutine per Subscribe).
    fn spawn_drain<S>(
        &self,
        topic: String,
        mut stream: S,
        handler: NatsHandlerFn,
    ) -> tokio::task::AbortHandle
    where
        S: futures_util::Stream<Item = async_nats::Message> + Send + Unpin + 'static,
    {
        tokio::spawn(async move {
            tracing::debug!(topic, "NATS: subscription started");
            while let Some(msg) = stream.next().await {
                let h = Arc::clone(&handler);
                let t = topic.clone();
                tokio::spawn(async move {
                    if let Err(e) = h(msg).await {
                        tracing::error!(topic = %t, error = %e, "NATS: handler error");
                    }
                });
            }
            tracing::debug!(topic, "NATS: subscription ended");
        })
        .abort_handle()
    }

    async fn cancel_subscription(&self, topic: &str) {
        let mut map = self.subscriptions.lock().await;
        if let Some(handle) = map.remove(topic) {
            handle.abort();
        }
    }
}

#[async_trait]
impl Publisher for NatsClient {
    async fn publish(
        &self,
        topic: &str,
        data: Bytes,
        attrs: HashMap<String, String>,
    ) -> Result<(), MessagingError> {
        // actor_id = None → "system" written by factory
        let nats_msg = self.factory.build_msg(topic, None, data, attrs)?;

        let inner = self.inner.clone();
        let subject = nats_msg.subject.clone();
        let payload = nats_msg.payload.clone();
        let headers = nats_msg.headers.clone();

        let pub_fn: NatsHandlerFn = Arc::new(move |_msg| {
            let inner = inner.clone();
            let subject = subject.clone();
            let payload = payload.clone();
            let headers = headers.clone();

            Box::pin(async move {
                if let Some(hdrs) = headers {
                    inner
                        .publish_with_headers(subject, hdrs, payload)
                        .await
                        .map_err(|e| MessagingError::Publish(e.to_string()))
                } else {
                    inner
                        .publish(subject, payload)
                        .await
                        .map_err(|e| MessagingError::Publish(e.to_string()))
                }
            })
        });

        let chained = apply_middleware("publish", pub_fn, &self.middlewares);

        // Pass a dummy message through (the real data is captured in the closure)
        chained(nats_msg).await
    }

    /// Drain the connection, aborting all active subscription tasks.
    async fn close(&self) -> Result<(), MessagingError> {
        let mut map = self.subscriptions.lock().await;
        for handle in map.values() {
            handle.abort();
        }
        map.clear();
        drop(map);

        self.inner.drain().await.map_err(|_| MessagingError::Closed)
    }
}

#[async_trait]
impl Subscriber for NatsClient {
    async fn subscribe(&self, topic: &str, handler: Handler) -> Result<(), MessagingError> {
        let subject = self.factory.subject(topic);

        let stream = self
            .inner
            .subscribe(subject.clone())
            .await
            .map_err(|e| MessagingError::Subscribe(e.to_string()))?;

        let transport_handler =
            apply_middleware("subscriber", self.wrap_handler(handler), &self.middlewares);

        let handle = self.spawn_drain(subject.clone(), stream, transport_handler);
        self.subscriptions.lock().await.insert(subject, handle);
        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<(), MessagingError> {
        let subject = self.factory.subject(topic);
        self.cancel_subscription(&subject).await;
        Ok(())
    }

    async fn close(&self) -> Result<(), MessagingError> {
        Publisher::close(self).await
    }
}

#[async_trait]
impl QueueSubscriber for NatsClient {
    /// async-nats exposes `queue_subscribe` as a `Subscriber` stream.
    async fn queue_subscribe(
        &self,
        topic: &str,
        group: &str,
        handler: Handler,
    ) -> Result<(), MessagingError> {
        let subject = self.factory.subject(topic);

        let stream = self
            .inner
            .queue_subscribe(subject.clone(), group.to_string())
            .await
            .map_err(|e| MessagingError::Subscribe(e.to_string()))?;

        let transport_handler = apply_middleware(
            "queue_subscribe",
            self.wrap_handler(handler),
            &self.middlewares,
        );

        let handle = self.spawn_drain(subject.clone(), stream, transport_handler);
        self.subscriptions.lock().await.insert(subject, handle);
        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<(), MessagingError> {
        Subscriber::unsubscribe(self, topic).await
    }

    async fn close(&self) -> Result<(), MessagingError> {
        Publisher::close(self).await
    }
}

#[async_trait]
impl Broker for NatsClient {
    async fn request<T, R>(
        &self,
        topic: &str,
        payload: &T,
        attrs: HashMap<String, String>,
        timeout: Duration,
    ) -> Result<R, MessagingError>
    where
        T: serde::Serialize + Send + Sync,
        R: serde::de::DeserializeOwned,
    {
        let data = serde_json::to_vec(payload)
            .map_err(|e| MessagingError::Serialization(e.to_string()))?;

        let msg = self
            .factory
            .build_msg(topic, None, Bytes::from(data), attrs)?;

        let subject = msg.subject.clone();

        let reply = tokio::time::timeout(
            timeout,
            self.inner
                .send_request(subject, async_nats::Request::new().payload(msg.payload)),
        )
        .await
        .map_err(|_| MessagingError::Request("request timed out".into()))?
        .map_err(|e| MessagingError::Request(e.to_string()))?;

        serde_json::from_slice(&reply.payload)
            .map_err(|e| MessagingError::Deserialization(e.to_string()))
    }

    async fn close(&self) -> Result<(), MessagingError> {
        Publisher::close(self).await
    }
}

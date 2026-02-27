use std::{future::Future, pin::Pin, sync::Arc};

use crate::MessagingError;
use crate::nats::headers::NatsHeaderExtractor;

/// A single NATS message handler at the transport level.
/// Distinct from `pubsub::Handler` (which is domain-level, returns `Option<Bytes>`).
pub type NatsHandlerFn = Arc<
    dyn Fn(async_nats::Message) -> Pin<Box<dyn Future<Output = Result<(), MessagingError>> + Send>>
        + Send
        + Sync,
>;

/// The inner function type for a middleware.
/// Takes (operation, inner_handler) → new_handler.
type MiddlewareInner = Arc<dyn Fn(Arc<str>, NatsHandlerFn) -> NatsHandlerFn + Send + Sync>;

/// A middleware wraps a `NatsHandlerFn` → `NatsHandlerFn`.
/// The `operation` label is one of: `"publish"`, `"subscribe"`, `"queue_subscribe"`, `"request"`.
#[derive(Clone)]
pub struct MiddlewareFn {
    /// Human-readable label shown in `{:?}` output — set at construction time.
    name: &'static str,
    inner: MiddlewareInner,
}

impl MiddlewareFn {
    /// Create a named middleware.
    ///
    /// ```rust  
    /// let mw = MiddlewareFn::new("tracing", |op, next| { ... });  
    /// ```
    pub fn new<F>(name: &'static str, f: F) -> Self
    where
        F: Fn(Arc<str>, NatsHandlerFn) -> NatsHandlerFn + Send + Sync + 'static,
    {
        Self {
            name,
            inner: Arc::new(f),
        }
    }

    /// Call the middleware, wrapping `handler` for `operation`.
    pub fn apply(&self, operation: Arc<str>, handler: NatsHandlerFn) -> NatsHandlerFn {
        (self.inner)(operation, handler)
    }
}

/// Debug shows the middleware name — not the opaque function pointer.
impl std::fmt::Debug for MiddlewareFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Middleware").field(&self.name).finish()
    }
}

/// Apply a slice of middlewares to a handler.
///
/// Middlewares are folded in **reverse** so that the first element in
/// `middlewares` is the outermost (first to run before the inner handler).
pub fn apply_middleware(
    operation: &str,
    handler: NatsHandlerFn,
    middlewares: &[MiddlewareFn],
) -> NatsHandlerFn {
    let op: Arc<str> = Arc::from(operation);
    middlewares
        .iter()
        .rev()
        .fold(handler, |inner, mw| mw.apply(Arc::clone(&op), inner))
}

/// Built-in: tracing middleware.
///
/// Creates a child span for every inbound message, linked to the upstream
/// trace via the `traceparent` header.
///
/// Usage: include in the middleware list passed to `NatsClient::new`.
pub fn tracing_middleware() -> MiddlewareFn {
    MiddlewareFn::new("tracing", |op, inner| {
        Arc::new(move |msg: async_nats::Message| {
            let inner = Arc::clone(&inner);
            let op = Arc::clone(&op);
            let subject = msg.subject.to_string();

            // Extract upstream context from headers
            let parent_cx = if let Some(headers) = &msg.headers {
                opentelemetry::global::get_text_map_propagator(|p| {
                    p.extract(&NatsHeaderExtractor(headers))
                })
            } else {
                opentelemetry::Context::new()
            };

            Box::pin(async move {
                // Create a child span linked to upstream trace
                use tracing::Instrument;
                use tracing_opentelemetry::OpenTelemetrySpanExt;

                let span = tracing::info_span!(
                    "nats.message",
                    otel.name = format!("{} {}", op, subject),
                    messaging.system = "nats",
                    messaging.operation = %op,
                    messaging.destination = %subject,
                );
                let _ = span.set_parent(parent_cx);

                inner(msg).instrument(span).await
            }) as Pin<Box<dyn Future<Output = Result<(), MessagingError>> + Send>>
        })
    })
}

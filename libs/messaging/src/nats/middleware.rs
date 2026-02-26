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

/// A middleware wraps a `NatsHandlerFn` → `NatsHandlerFn`.
/// The `operation` label is one of: `"publish"`, `"subscribe"`, `"queue_subscribe"`, `"request"`.
pub type MiddlewareFn = Arc<
    dyn Fn(
            Arc<str>, // operation
            NatsHandlerFn,
        ) -> NatsHandlerFn
        + Send
        + Sync,
>;

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
        .fold(handler, |inner, mw| mw(Arc::clone(&op), inner))
}

/// Built-in: tracing middleware.
///
/// Creates a child span for every inbound message, linked to the upstream
/// trace via the `traceparent` header.
///
/// Usage: include in the middleware list passed to `NatsClient::new`.
pub fn tracing_middleware() -> MiddlewareFn {
    Arc::new(|operation: Arc<str>, inner: NatsHandlerFn| {
        let op = Arc::clone(&operation);
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

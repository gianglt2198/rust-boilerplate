use axum::{extract::Request, middleware::Next, response::Response};
use opentelemetry::{
    KeyValue,
    trace::{FutureExt, SpanKind, TraceContextExt, Tracer},
};

use crate::middlewares::RequestId;

use ro_common::id::generate_nanoid;
use ro_telemetry::{extract_context_from_request, tracer::get_tracer};

/// Middleware to add tracing to all requests
pub async fn tracing_middleware(request: Request, next: Next) -> Response {
    let parent_cx = extract_context_from_request(&request);
    let tracer = get_tracer();
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|v| v.0.clone())
        .unwrap_or_else(generate_nanoid);

    let span = tracer
        .span_builder(format!("{} {}", request.method(), request.uri().path()))
        .with_attributes([KeyValue::new("cid", request_id)])
        .with_kind(SpanKind::Server)
        .start_with_context(tracer, &parent_cx);
    let cx = parent_cx.with_span(span);

    next.run(request).with_context(cx).await
}

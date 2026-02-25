pub mod meter;
pub mod tracer;

use axum::extract::Request;
use opentelemetry::{Context, global};
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

// Utility function to extract the context from the incoming request headers
pub fn extract_context_from_request(req: &Request) -> Context {
    global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    })
}

// Utility function to inject the trace to the request headers
#[allow(dead_code)]
pub fn inject_trace_headers(req: &mut Request) {
    let current_span = Span::current();
    let context = current_span.context();

    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&context, &mut HeaderInjector(req.headers_mut()));
    });
}

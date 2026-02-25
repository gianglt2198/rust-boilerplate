use axum::{
    body::HttpBody,
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use opentelemetry::KeyValue;

use ro_telemetry::meter::http::{
    ACTIVE_REQUESTS, ERROR_COUNT_4XX, ERROR_COUNT_5XX, REQUEST_COUNT, REQUEST_LATENCY,
    RESPONSE_SIZE,
};

/// Middleware to add metric to all requests
pub async fn metric_middleware(req: Request, next: Next) -> Response {
    let start_time = std::time::Instant::now();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_owned())
        .unwrap_or_else(|| req.uri().path().to_owned());

    let attributes = vec![
        KeyValue::new("method", req.method().to_string()),
        KeyValue::new("route", route),
    ];

    // Increment active requests
    ACTIVE_REQUESTS.add(1.0, &attributes);

    // Proceed with the request
    let response = next.run(req).await;

    let duration = start_time.elapsed().as_secs_f64();

    // Record request latency
    REQUEST_LATENCY.record(duration, &attributes);

    // Record total requests
    REQUEST_COUNT.add(1, &attributes);

    // Get response status code
    let status = response.status();

    // Record error counters based on status code
    if status.as_u16() >= 400 && status.as_u16() < 500 {
        ERROR_COUNT_4XX.add(1, &attributes);
    } else if status.as_u16() >= 500 {
        ERROR_COUNT_5XX.add(1, &attributes);
    }

    // Measure response size
    let size = response.body().size_hint().upper().unwrap_or(0);
    RESPONSE_SIZE.record(size as f64, &attributes);

    // Decrement active requests
    ACTIVE_REQUESTS.add(-1.0, &attributes);

    response
}

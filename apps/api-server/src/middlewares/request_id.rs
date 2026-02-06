use axum::{extract::Request, http::header, middleware::Next, response::Response};

use ro_common::id::generate_nanoid;

use crate::middlewares::RequestId;

const REQUEST_ID_HEADER: &str = "x-request-id";

/// Middleware to add request ID to all requests
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or_else(generate_nanoid);

    // Add request id to request
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    // Log info
    tracing::info!(
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
        "Incoming request",
    );

    let mut response = next.run(request).await;

    // Add X-Request-ID header to response
    response.headers_mut().insert(
        header::HeaderName::from_static(REQUEST_ID_HEADER),
        header::HeaderValue::from_str(&request_id.to_string()).unwrap(),
    );

    // Log response
    tracing::info!(
        request_id = %request_id,
        status = %response.status(),
        "Request completed"
    );

    response
}

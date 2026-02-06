pub mod metrics;
pub mod request_id;
pub mod tracing;

// A type to hold the request ID in the request extensions
#[derive(Clone, Debug)]
pub struct RequestId(String);

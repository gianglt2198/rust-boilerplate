use once_cell::sync::Lazy;

use crate::meter::get_meter;

// Define metrics as static to initialize once
pub static REQUEST_LATENCY: Lazy<opentelemetry::metrics::Histogram<f64>> =
    Lazy::new(|| get_meter().f64_histogram("http.request.duration").build());

pub static REQUEST_COUNT: Lazy<opentelemetry::metrics::Counter<u64>> =
    Lazy::new(|| get_meter().u64_counter("http.requests.total").build());

pub static ERROR_COUNT_4XX: Lazy<opentelemetry::metrics::Counter<u64>> =
    Lazy::new(|| get_meter().u64_counter("http.errors.4xx").build());

pub static ERROR_COUNT_5XX: Lazy<opentelemetry::metrics::Counter<u64>> =
    Lazy::new(|| get_meter().u64_counter("http.errors.5xx").build());

pub static RESPONSE_SIZE: Lazy<opentelemetry::metrics::Histogram<f64>> =
    Lazy::new(|| get_meter().f64_histogram("http.response.size").build());

pub static ACTIVE_REQUESTS: Lazy<opentelemetry::metrics::UpDownCounter<f64>> = Lazy::new(|| {
    get_meter()
        .f64_up_down_counter("http.request.active")
        .build()
});

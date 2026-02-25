pub mod http;
pub mod system;

use std::{sync::OnceLock, time::Duration};

use opentelemetry::{global, metrics::Meter};
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider};
use tokio::time::interval;
// use opentelemetry_stdout::MetricExporter;

use crate::meter::system::SystemMetrics;

static METER: OnceLock<Meter> = OnceLock::new();

pub fn init_meter(name: String, endpoint: String) -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("Failed to create meter exporter");

    let resource = Resource::builder().with_service_name(name.clone()).build();

    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(resource)
        .build();
    global::set_meter_provider(provider.clone());

    METER.get_or_init(|| global::meter(Box::leak(Box::new(name.clone()))));

    provider
}

// Function to collect system metrics periodically
pub async fn collect_system_metrics(interval_secs: u64) {
    let mut interval = interval(Duration::from_secs(interval_secs));
    let system_metrics = SystemMetrics::new();
    loop {
        interval.tick().await;
        {
            system_metrics.update_stats().await;
        }
    }
}

pub fn get_meter() -> &'static Meter {
    METER.get_or_init(|| global::meter("meter".to_string().clone().leak()))
}

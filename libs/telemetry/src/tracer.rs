use std::time::Duration;

use opentelemetry::{
    Context, KeyValue,
    baggage::BaggageExt,
    global::{self},
    propagation::TextMapCompositePropagator,
    trace::Span,
};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    error::OTelSdkResult,
    propagation::{BaggagePropagator, TraceContextPropagator},
    trace::{SdkTracerProvider, SpanProcessor},
};
// use opentelemetry_stdout::SpanExporter;

use ro_config::definition::AppConfig;

pub fn init_tracer(cfg: AppConfig) -> SdkTracerProvider {
    let baggage_propagator = BaggagePropagator::new();
    let trace_context_propagator = TraceContextPropagator::new();
    let composite_propagator = TextMapCompositePropagator::new(vec![
        Box::new(baggage_propagator),
        Box::new(trace_context_propagator),
    ]);

    global::set_text_map_propagator(composite_propagator);

    let resource = Resource::builder()
        .with_service_name(cfg.common.name.clone())
        .build();

    // Setup tracerprovider with stdout exporter
    // that prints the spans to stdout.
    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_span_processor(EnrichWithBaggageSpanProcessor)
        .with_batch_exporter(
            SpanExporter::builder()
                .with_tonic()
                .with_endpoint(cfg.otel.exporter.endpoint)
                .build()
                .expect("Failed to create span exporter"),
        )
        .build();

    global::set_tracer_provider(provider.clone());
    provider
}

/// A custom span processor that enriches spans with baggage attributes. Baggage
/// information is not added automatically without this processor.
#[derive(Debug)]
struct EnrichWithBaggageSpanProcessor;
impl SpanProcessor for EnrichWithBaggageSpanProcessor {
    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }

    fn on_start(&self, span: &mut opentelemetry_sdk::trace::Span, cx: &Context) {
        for (kk, vv) in cx.baggage().iter() {
            span.set_attribute(KeyValue::new(kk.clone(), vv.0.clone()));
        }
    }

    fn on_end(&self, _span: opentelemetry_sdk::trace::SpanData) {}
}

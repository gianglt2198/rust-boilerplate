use std::{collections::HashMap, str::FromStr, sync::Arc};

use async_nats::header::{HeaderMap, HeaderName, HeaderValue};
use bytes::Bytes;
use chrono::{SecondsFormat, Utc};
use opentelemetry::global;
use opentelemetry::propagation::Extractor;
use ro_config::config::nats::NatsConfig;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::Message;
use crate::MessagingError;
use crate::nats::headers::NatsHeaderExtractor;
use crate::nats::headers::NatsHeaderInjector;

pub const HEADER_USER_ID: &str = "user_id";
pub const HEADER_FROM: &str = "from";
pub const HEADER_START_TIME: &str = "start_time";

#[derive(Debug)]
pub struct MessageFactory {
    name: String,
    cfg: Arc<NatsConfig>,
}

impl MessageFactory {
    pub fn new(name: String, config: Arc<NatsConfig>) -> Self {
        Self { name, cfg: config }
    }

    pub fn subject(&self, pattern: &str) -> String {
        if self.cfg.base_path.is_empty() {
            pattern.to_string()
        } else {
            format!("{}.{}", self.cfg.base_path, pattern)
        }
    }

    /// Build a `nats::Message` ready for sending.
    ///
    /// Headers written:
    ///   - `user_id`    — from `actor_id` (replaces Go's `GetUserIDFromCtx`)
    ///   - `from`       — config.name
    ///   - `start_time` — RFC3339 nanoseconds
    ///   - `traceparent`/ `tracestate` — injected from `tracing::Span::current()`
    ///   - any extra `attrs` provided by the caller
    pub fn build_msg(
        &self,
        pattern: &str,
        actor_id: Option<&str>,
        data: Bytes,
        attrs: HashMap<String, String>,
    ) -> Result<async_nats::Message, MessagingError> {
        let subject = self.subject(pattern);
        let headers = self.build_headers(actor_id, attrs)?;

        Ok(async_nats::Message {
            subject: subject.into(),
            reply: None,
            payload: data,
            headers: Some(headers),
            status: None,
            description: None,
            length: 0,
        })
    }

    /// Convert an incoming `async_nats::Message` into a `pubsub::Message`.
    ///
    /// All NATS headers become `attrs` (including traceparent so the caller
    /// can extract the parent span context for their own child span).
    ///
    pub fn read_message(&self, msg: async_nats::Message) -> Result<Message, MessagingError> {
        let topic = msg.subject.to_string();
        let data = msg.payload;

        // Extract all headers into attrs map
        let attrs: HashMap<String, String> = msg
            .headers
            .as_ref()
            .map(|h| {
                let extractor = NatsHeaderExtractor(h);
                extractor
                    .keys()
                    .into_iter()
                    .filter_map(|k| extractor.get(k).map(|v| (k.to_string(), v.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Message { topic, data, attrs })
    }

    /// Extract the parent OTel context from an inbound message's headers.
    ///
    /// Use this inside a subscriber handler to link the child span to
    /// the upstream trace:
    /// ```rust  
    /// let parent_cx = factory.extract_trace_context(&msg);  
    /// let span = tracer.start_with_context("nats.consume", &parent_cx);  
    /// ```
    pub fn extract_trace_context(&self, msg: &async_nats::Message) -> opentelemetry::Context {
        if let Some(headers) = &msg.headers {
            global::get_text_map_propagator(|propagator| {
                propagator.extract(&NatsHeaderExtractor(headers))
            })
        } else {
            opentelemetry::Context::new()
        }
    }

    fn build_headers(
        &self,
        actor_id: Option<&str>,
        attrs: HashMap<String, String>,
    ) -> Result<HeaderMap, MessagingError> {
        let mut headers = HeaderMap::new();

        self.insert_header(&mut headers, HEADER_USER_ID, actor_id.unwrap_or("system"))?;
        self.insert_header(&mut headers, HEADER_FROM, &self.name.clone())?;
        self.insert_header(
            &mut headers,
            HEADER_START_TIME,
            &Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true),
        )?;

        let otel_cx = tracing::Span::current().context();
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&otel_cx, &mut NatsHeaderInjector(&mut headers));
        });

        for (k, v) in &attrs {
            let _ = self.insert_header(&mut headers, k, v);
        }

        Ok(headers)
    }

    fn insert_header(
        &self,
        map: &mut HeaderMap,
        key: &str,
        value: &str,
    ) -> Result<(), MessagingError> {
        let name = HeaderName::from_str(key)
            .map_err(|e| MessagingError::Publish(format!("invalid header name {key}: {e}")))?;
        let val = HeaderValue::from(value);
        map.insert(name, val);
        Ok(())
    }
}

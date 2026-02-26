use std::str::FromStr;

use async_nats::header::{HeaderMap, HeaderName, HeaderValue};
use opentelemetry::propagation::{Extractor, Injector};

pub struct NatsHeaderInjector<'a>(pub &'a mut HeaderMap);

impl Injector for NatsHeaderInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(name), val) = (HeaderName::from_str(key), HeaderValue::from(value)) {
            self.0.insert(name, val);
        }
    }
}

pub struct NatsHeaderExtractor<'a>(pub &'a HeaderMap);

impl Extractor for NatsHeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| Some(v.as_str()))
    }

    fn keys(&self) -> Vec<&str> {
        self.0.iter().map(|(k, _)| k.as_ref()).collect()
    }
}

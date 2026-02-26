use std::collections::HashMap;

use bytes::Bytes;
use serde::de::DeserializeOwned;

use crate::MessagingError;

/// A transport-agnostic message envelope.
///
/// On outbound (publish), `attrs` are written as transport headers.
/// On inbound (subscribe), `attrs` are populated from transport headers.
#[derive(Debug, Clone)]
pub struct Message {
    /// Resolved subject/topic (base_path already applied on inbound)
    pub topic: String,
    /// Raw payload bytes
    pub data: Bytes,
    /// Key/value metadata (NATS headers, trace IDs, user_id, etc.)
    pub attrs: HashMap<String, String>,
}

impl Message {
    pub fn new(topic: impl Into<String>, data: impl Into<Bytes>) -> Self {
        Self {
            topic: topic.into(),
            data: data.into(),
            attrs: HashMap::new(),
        }
    }

    pub fn with_attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.insert(key.into(), value.into());
        self
    }

    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).map(|s| s.as_str())
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T, MessagingError> {
        serde_json::from_slice(&self.data)
            .map_err(|e| MessagingError::Deserialization(e.to_string()))
    }

    pub fn from_json<T: serde::Serialize>(
        topic: impl Into<String>,
        payload: &T,
    ) -> Result<Self, MessagingError> {
        let data = serde_json::to_vec(payload)
            .map_err(|e| MessagingError::Serialization(e.to_string()))?;
        Ok(Self::new(topic, Bytes::from(data)))
    }
}

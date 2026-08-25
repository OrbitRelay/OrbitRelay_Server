//! Extensible object payloads carried by actions and events.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A JSON-compatible payload whose top level is always an object.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Payload(Map<String, Value>);

impl Payload {
    /// Creates an empty object payload.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a value and returns the previous value for the key, if any.
    pub fn insert(&mut self, key: impl Into<String>, value: Value) -> Option<Value> {
        self.0.insert(key.into(), value)
    }

    /// Returns a payload value by key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    /// Iterates over the payload fields.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.0.iter().map(|(key, value)| (key.as_str(), value))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Payload;

    #[test]
    fn inserts_and_reads_payload_values() {
        let mut payload = Payload::new();
        payload.insert("stroke", json!({ "color": "red" }));

        assert_eq!(payload.get("stroke"), Some(&json!({ "color": "red" })));
        assert_eq!(payload.iter().count(), 1);
    }

    #[test]
    fn round_trips_as_a_json_object() {
        let mut payload = Payload::new();
        payload.insert("x", json!(12));

        let encoded = serde_json::to_string(&payload).expect("payload should serialize");
        let decoded: Payload = serde_json::from_str(&encoded).expect("payload should deserialize");

        assert_eq!(decoded, payload);
        assert!(serde_json::from_str::<Payload>("[]").is_err());
    }
}

//! Generic, business-neutral metadata.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// An ordered collection of string key-value metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Metadata(BTreeMap<String, String>);

impl Metadata {
    /// Creates empty metadata.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of metadata entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether there are no metadata entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Inserts a value and returns the previous value for the key, if any.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.0.insert(key.into(), value.into())
    }

    /// Returns a metadata value by key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    /// Removes a metadata value and returns it, if present.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.0.remove(key)
    }

    /// Iterates over metadata entries in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::Metadata;

    #[test]
    fn inserts_and_removes_values() {
        let mut metadata = Metadata::new();

        assert!(metadata.is_empty());
        assert_eq!(metadata.insert("origin", "test"), None);
        assert_eq!(
            metadata.insert("origin", "updated"),
            Some("test".to_owned())
        );
        assert_eq!(metadata.get("origin"), Some("updated"));
        assert_eq!(metadata.len(), 1);
        assert_eq!(metadata.remove("origin"), Some("updated".to_owned()));
        assert!(metadata.is_empty());
    }

    #[test]
    fn serializes_and_deserializes() {
        let mut metadata = Metadata::new();
        metadata.insert("b", "second");
        metadata.insert("a", "first");

        let encoded = serde_json::to_string(&metadata).expect("metadata should serialize");
        let decoded: Metadata =
            serde_json::from_str(&encoded).expect("metadata should deserialize");

        assert_eq!(metadata, decoded);
        assert_eq!(
            decoded.iter().collect::<Vec<_>>(),
            vec![("a", "first"), ("b", "second")]
        );
    }
}

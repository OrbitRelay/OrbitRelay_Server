//! Opaque identifiers shared by higher-level OrbitRelay domains.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::CoreError;

/// A random, opaque identifier for a core entity.
///
/// The core crate deliberately does not know which domain object an identifier
/// belongs to. Higher-level crates can wrap this type in domain-specific IDs.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Creates a new random UUID v4 identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wraps an existing UUID without changing it.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the wrapped UUID by reference.
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Parses a UUID string into an entity identifier.
    pub fn parse(value: &str) -> Result<Self, CoreError> {
        Ok(Self(Uuid::parse_str(value)?))
    }
}

impl FromStr for EntityId {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::EntityId;

    #[test]
    fn creates_distinct_ids() {
        let first = EntityId::new();
        let second = EntityId::new();

        assert_ne!(first, second);
        assert_eq!(first.as_uuid().get_version_num(), 4);
    }

    #[test]
    fn serializes_and_deserializes() {
        let id = EntityId::new();
        let encoded = serde_json::to_string(&id).expect("entity id should serialize");
        let decoded: EntityId =
            serde_json::from_str(&encoded).expect("entity id should deserialize");

        assert_eq!(id, decoded);
    }

    #[test]
    fn parses_and_displays() {
        let source = "550e8400-e29b-41d4-a716-446655440000";
        let id = EntityId::parse(source).expect("valid UUID should parse");

        assert_eq!(id.to_string(), source);
        assert_eq!(
            source.parse::<EntityId>().expect("valid UUID should parse"),
            id
        );
    }
}

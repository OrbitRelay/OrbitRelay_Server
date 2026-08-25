//! Type-safe node identities.

use std::{fmt, str::FromStr};

use orbitrelay_core::{CoreError, EntityId};
use serde::{Deserialize, Serialize};

/// Identifies one OrbitRelay service instance.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(EntityId);

impl NodeId {
    /// Creates a new random node identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(EntityId::new())
    }

    /// Wraps an existing core entity identifier.
    #[must_use]
    pub const fn from_entity_id(value: EntityId) -> Self {
        Self(value)
    }

    /// Returns the wrapped core entity identifier.
    #[must_use]
    pub const fn as_entity_id(&self) -> &EntityId {
        &self.0
    }

    /// Parses a node identifier from a UUID string.
    pub fn parse(value: &str) -> Result<Self, CoreError> {
        Ok(Self(EntityId::parse(value)?))
    }
}

impl FromStr for NodeId {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::NodeId;

    #[test]
    fn creates_distinct_node_ids() {
        let first = NodeId::new();
        let second = NodeId::new();

        assert_ne!(first, second);
        assert_eq!(first.as_entity_id().as_uuid().get_version_num(), 4);
    }

    #[test]
    fn parses_displays_and_serializes_node_ids() {
        let source = "550e8400-e29b-41d4-a716-446655440000";
        let id: NodeId = source.parse().expect("valid UUID should parse");
        let encoded = serde_json::to_string(&id).expect("node ID should serialize");
        let decoded: NodeId = serde_json::from_str(&encoded).expect("node ID should deserialize");

        assert_eq!(id.to_string(), source);
        assert_eq!(decoded, id);
    }
}

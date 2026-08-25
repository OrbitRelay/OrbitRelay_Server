//! Serializable node descriptions.

use std::collections::BTreeSet;

use orbitrelay_core::Metadata;
use serde::{Deserialize, Serialize};

use crate::{Capability, NodeError, NodeId, NodeState};

/// A snapshot describing one node and the capabilities it advertises.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Node {
    id: NodeId,
    metadata: Metadata,
    state: NodeState,
    capabilities: BTreeSet<Capability>,
}

impl Node {
    /// Creates a node snapshot, sorting and deduplicating its capabilities.
    #[must_use]
    pub fn new(
        id: NodeId,
        metadata: Metadata,
        state: NodeState,
        capabilities: impl IntoIterator<Item = Capability>,
    ) -> Self {
        Self {
            id,
            metadata,
            state,
            capabilities: capabilities.into_iter().collect(),
        }
    }

    /// Returns the node identifier.
    #[must_use]
    pub const fn id(&self) -> &NodeId {
        &self.id
    }

    /// Returns the node metadata.
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Returns the node lifecycle state.
    #[must_use]
    pub const fn state(&self) -> NodeState {
        self.state
    }

    /// Returns the capabilities in deterministic lexical order.
    #[must_use]
    pub const fn capabilities(&self) -> &BTreeSet<Capability> {
        &self.capabilities
    }

    pub(crate) fn validate(&self) -> Result<(), NodeError> {
        if let Some(capability) = self.capabilities.iter().find(|capability| {
            capability.as_str().trim().is_empty()
                || capability.as_str().trim() != capability.as_str()
        }) {
            return Err(NodeError::InvalidNode {
                reason: format!(
                    "capability `{capability}` must be non-empty and contain no surrounding whitespace"
                ),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::Metadata;

    use super::Node;
    use crate::{Capability, NodeId, NodeState};

    #[test]
    fn serializes_and_deserializes_a_node_snapshot() {
        let mut metadata = Metadata::new();
        metadata.insert("region", "ap-southeast");
        let node = Node::new(
            NodeId::new(),
            metadata,
            NodeState::Ready,
            [
                Capability::new("sync"),
                Capability::new("storage"),
                Capability::new("sync"),
            ],
        );

        let encoded = serde_json::to_string(&node).expect("node should serialize");
        let decoded: Node = serde_json::from_str(&encoded).expect("node should deserialize");

        assert_eq!(decoded, node);
        assert_eq!(decoded.state(), NodeState::Ready);
        assert_eq!(decoded.capabilities().len(), 2);
    }
}

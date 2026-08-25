//! Node registration and discovery abstraction.

use async_trait::async_trait;

use crate::{Node, NodeError, NodeId};

/// Registers and discovers current node description snapshots.
#[async_trait]
pub trait NodeRegistry: Send + Sync {
    /// Creates or replaces the snapshot for a node identifier.
    async fn register(&self, node: Node) -> Result<(), NodeError>;

    /// Removes a node if present; absence is treated as success.
    async fn unregister(&self, node_id: &NodeId) -> Result<(), NodeError>;

    /// Gets the current snapshot for a node identifier.
    async fn get(&self, node_id: &NodeId) -> Result<Option<Node>, NodeError>;

    /// Lists all currently registered nodes in identifier order.
    async fn list(&self) -> Result<Vec<Node>, NodeError>;
}

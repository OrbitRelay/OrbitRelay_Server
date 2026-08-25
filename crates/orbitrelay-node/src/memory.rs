//! Thread-safe in-memory node registry implementation.

use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use async_trait::async_trait;

use crate::{Node, NodeError, NodeId, NodeRegistry};

/// A cloneable, thread-safe in-memory node registry.
#[derive(Clone, Default)]
pub struct MemoryNodeRegistry {
    nodes: Arc<RwLock<BTreeMap<NodeId, Node>>>,
}

impl MemoryNodeRegistry {
    /// Creates an empty in-memory node registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn read_nodes(&self) -> Result<RwLockReadGuard<'_, BTreeMap<NodeId, Node>>, NodeError> {
        self.nodes.read().map_err(|_| NodeError::RegistryFailure {
            message: "memory registry read lock is poisoned".to_owned(),
        })
    }

    fn write_nodes(&self) -> Result<RwLockWriteGuard<'_, BTreeMap<NodeId, Node>>, NodeError> {
        self.nodes.write().map_err(|_| NodeError::RegistryFailure {
            message: "memory registry write lock is poisoned".to_owned(),
        })
    }
}

#[async_trait]
impl NodeRegistry for MemoryNodeRegistry {
    async fn register(&self, node: Node) -> Result<(), NodeError> {
        node.validate()?;
        self.write_nodes()?.insert(node.id().clone(), node);
        Ok(())
    }

    async fn unregister(&self, node_id: &NodeId) -> Result<(), NodeError> {
        self.write_nodes()?.remove(node_id);
        Ok(())
    }

    async fn get(&self, node_id: &NodeId) -> Result<Option<Node>, NodeError> {
        Ok(self.read_nodes()?.get(node_id).cloned())
    }

    async fn list(&self) -> Result<Vec<Node>, NodeError> {
        Ok(self.read_nodes()?.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use orbitrelay_core::{EntityId, Metadata};

    use super::MemoryNodeRegistry;
    use crate::{Capability, Node, NodeError, NodeId, NodeRegistry, NodeState};

    fn node(id: NodeId, state: NodeState) -> Node {
        Node::new(id, Metadata::new(), state, [Capability::new("sync")])
    }

    #[tokio::test]
    async fn registers_and_gets_a_node() {
        let registry = MemoryNodeRegistry::new();
        let node = node(NodeId::new(), NodeState::Starting);

        registry
            .register(node.clone())
            .await
            .expect("registration should succeed");

        assert_eq!(
            registry.get(node.id()).await.expect("get should succeed"),
            Some(node)
        );
    }

    #[tokio::test]
    async fn registration_upserts_a_node_snapshot() {
        let registry = MemoryNodeRegistry::new();
        let id = NodeId::new();
        registry
            .register(node(id.clone(), NodeState::Starting))
            .await
            .expect("initial registration should succeed");
        registry
            .register(node(id.clone(), NodeState::Ready))
            .await
            .expect("upsert should succeed");

        let registered = registry
            .get(&id)
            .await
            .expect("get should succeed")
            .expect("node should remain registered");
        assert_eq!(registered.state(), NodeState::Ready);
        assert_eq!(registry.list().await.expect("list should succeed").len(), 1);
    }

    #[tokio::test]
    async fn unregister_is_idempotent() {
        let registry = MemoryNodeRegistry::new();
        let id = NodeId::new();
        registry
            .unregister(&id)
            .await
            .expect("missing node removal should succeed");
        registry
            .register(node(id.clone(), NodeState::Ready))
            .await
            .expect("registration should succeed");
        registry
            .unregister(&id)
            .await
            .expect("registered node removal should succeed");
        registry
            .unregister(&id)
            .await
            .expect("repeated removal should succeed");

        assert!(registry
            .get(&id)
            .await
            .expect("get should succeed")
            .is_none());
    }

    #[tokio::test]
    async fn lists_nodes_in_identifier_order() {
        let registry = MemoryNodeRegistry::new();
        let first = NodeId::from_entity_id(
            EntityId::parse("00000000-0000-4000-8000-000000000001")
                .expect("node ID should be valid"),
        );
        let second = NodeId::from_entity_id(
            EntityId::parse("00000000-0000-4000-8000-000000000002")
                .expect("node ID should be valid"),
        );

        registry
            .register(node(second.clone(), NodeState::Ready))
            .await
            .expect("registration should succeed");
        registry
            .register(node(first.clone(), NodeState::Ready))
            .await
            .expect("registration should succeed");

        let ids = registry
            .list()
            .await
            .expect("list should succeed")
            .into_iter()
            .map(|node| node.id().clone())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec![first, second]);
    }

    #[tokio::test]
    async fn rejects_invalid_node_capabilities() {
        let registry = MemoryNodeRegistry::new();
        let invalid = Node::new(
            NodeId::new(),
            Metadata::new(),
            NodeState::Ready,
            [Capability::new("  ")],
        );

        let error = registry
            .register(invalid)
            .await
            .expect_err("invalid node should be rejected");
        assert!(matches!(error, NodeError::InvalidNode { .. }));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn supports_concurrent_registry_access() {
        let registry = Arc::new(MemoryNodeRegistry::new());
        let mut tasks = Vec::new();

        for _ in 0..32 {
            let registry = registry.clone();
            tasks.push(tokio::spawn(async move {
                let node = node(NodeId::new(), NodeState::Ready);
                let id = node.id().clone();
                registry.register(node).await?;
                registry.get(&id).await
            }));
        }

        for task in tasks {
            assert!(task
                .await
                .expect("registry task should complete")
                .expect("registry access should succeed")
                .is_some());
        }

        assert_eq!(
            registry.list().await.expect("list should succeed").len(),
            32
        );
    }
}

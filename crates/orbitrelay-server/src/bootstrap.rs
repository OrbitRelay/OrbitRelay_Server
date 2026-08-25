//! Dependency construction and node lifecycle management.

use std::sync::Arc;

use orbitrelay_node::{Capability, MemoryNodeRegistry, Node, NodeId, NodeRegistry, NodeState};
use orbitrelay_runtime::{
    ActionAuthorizer, EventPipeline, HandlerRegistry, Runtime, RuntimeContext, SystemClock,
};
use orbitrelay_storage::{EventStore, MemoryEventStore};
use orbitrelay_sync::{EventBus, MemoryEventBus};

use crate::{PipelineAdapter, ServerConfig, ServerContext, ServerError, ServerLifecycle};

/// Creates the in-process dependencies and manages the local node lifecycle.
#[derive(Clone)]
pub struct Bootstrap {
    config: ServerConfig,
    authorizer: Arc<dyn ActionAuthorizer>,
}

impl Bootstrap {
    /// Creates a bootstrapper with an externally supplied authorizer.
    #[must_use]
    pub fn new(config: ServerConfig, authorizer: Arc<dyn ActionAuthorizer>) -> Self {
        Self { config, authorizer }
    }

    /// Creates the memory-backed context and registers the local node as ready.
    pub async fn build(&self) -> Result<ServerContext, ServerError> {
        self.config.validate()?;

        let event_store: Arc<dyn EventStore> = Arc::new(MemoryEventStore::new());
        let event_bus: Arc<dyn EventBus> = Arc::new(
            MemoryEventBus::with_queue_capacity(self.config.subscription_queue_capacity())
                .map_err(|_| ServerError::bootstrap("event bus initialization failed"))?,
        );
        let node_registry: Arc<dyn NodeRegistry> = Arc::new(MemoryNodeRegistry::new());

        self.build_with(event_store, event_bus, node_registry).await
    }

    async fn build_with(
        &self,
        event_store: Arc<dyn EventStore>,
        event_bus: Arc<dyn EventBus>,
        node_registry: Arc<dyn NodeRegistry>,
    ) -> Result<ServerContext, ServerError> {
        self.config.validate()?;

        let node_id = self.config.node_id().cloned().unwrap_or_else(NodeId::new);
        let lifecycle = ServerLifecycle::new();
        lifecycle.start()?;
        self.register_state_with(node_registry.as_ref(), &node_id, NodeState::Starting)
            .await?;
        let pipeline: Arc<dyn EventPipeline> =
            Arc::new(PipelineAdapter::new(event_store.clone(), event_bus.clone()));
        let runtime = Arc::new(Runtime::new(
            Arc::new(HandlerRegistry::new()),
            RuntimeContext::new(Arc::new(SystemClock), self.authorizer.clone()),
            pipeline,
        ));
        let ready_node = self.node(&node_id, NodeState::Ready);
        node_registry
            .register(ready_node.clone())
            .await
            .map_err(|_| ServerError::node_lifecycle("node state registration failed"))?;
        lifecycle.ready()?;
        let context = ServerContext::new_composed(
            ready_node,
            lifecycle,
            runtime,
            event_store,
            event_bus,
            node_registry,
        );

        Ok(context)
    }

    /// Transitions the local node through shutdown states and unregisters it.
    pub async fn shutdown(&self, context: &ServerContext) -> Result<(), ServerError> {
        context.shutdown().await
    }

    async fn register_state_with(
        &self,
        node_registry: &dyn NodeRegistry,
        node_id: &NodeId,
        state: NodeState,
    ) -> Result<(), ServerError> {
        node_registry
            .register(self.node(node_id, state))
            .await
            .map_err(|_| ServerError::node_lifecycle("node state registration failed"))
    }

    fn node(&self, node_id: &NodeId, state: NodeState) -> Node {
        Node::new(
            node_id.clone(),
            self.config.node_metadata().clone(),
            state,
            [
                Capability::new("runtime"),
                Capability::new("storage"),
                Capability::new("sync"),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use orbitrelay_node::{Node, NodeError, NodeId, NodeRegistry, NodeState};
    use orbitrelay_protocol::Action;
    use orbitrelay_runtime::{ActionAuthorizer, AuthorizationError};
    use orbitrelay_storage::{EventStore, MemoryEventStore};
    use orbitrelay_sync::{EventBus, MemoryEventBus};

    use super::Bootstrap;
    use crate::ServerConfig;

    struct TestAuthorizer;

    #[async_trait]
    impl ActionAuthorizer for TestAuthorizer {
        async fn authorize(&self, _action: &Action) -> Result<(), AuthorizationError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingRegistry {
        states: Mutex<Vec<NodeState>>,
        node: Mutex<Option<Node>>,
    }

    impl RecordingRegistry {
        fn states(&self) -> Vec<NodeState> {
            self.states
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        }
    }

    #[async_trait]
    impl NodeRegistry for RecordingRegistry {
        async fn register(&self, node: Node) -> Result<(), NodeError> {
            self.states
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(node.state());
            *self
                .node
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(node);
            Ok(())
        }

        async fn unregister(&self, _node_id: &NodeId) -> Result<(), NodeError> {
            *self
                .node
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            Ok(())
        }

        async fn get(&self, node_id: &NodeId) -> Result<Option<Node>, NodeError> {
            Ok(self
                .node
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref()
                .filter(|node| node.id() == node_id)
                .cloned())
        }

        async fn list(&self) -> Result<Vec<Node>, NodeError> {
            Ok(self
                .node
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .iter()
                .cloned()
                .collect())
        }
    }

    #[tokio::test]
    async fn builds_context_and_registers_ready_node() {
        let bootstrap = Bootstrap::new(ServerConfig::default(), Arc::new(TestAuthorizer));
        let context = bootstrap.build().await.expect("bootstrap should succeed");

        let node = context
            .node_registry()
            .get(context.node_id())
            .await
            .expect("node lookup should succeed")
            .expect("node should be registered");
        assert_eq!(node.state(), NodeState::Ready);
        assert_eq!(context.lifecycle().state(), crate::LifecycleState::Ready);
        assert_eq!(context.health().state(), crate::HealthState::Ready);
        assert!(context.health().is_ready());
        assert_eq!(
            context
                .node_registry()
                .list()
                .await
                .expect("list should succeed")
                .len(),
            1
        );
        assert!(!context
            .runtime()
            .registry()
            .contains(&orbitrelay_protocol::ActionType::new("unregistered.action")));
    }

    #[tokio::test]
    async fn shuts_down_and_unregisters_node() {
        let bootstrap = Bootstrap::new(ServerConfig::default(), Arc::new(TestAuthorizer));
        let registry = Arc::new(RecordingRegistry::default());
        let event_store: Arc<dyn EventStore> = Arc::new(MemoryEventStore::new());
        let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new());
        let context = bootstrap
            .build_with(event_store, event_bus, registry.clone())
            .await
            .expect("bootstrap should succeed");

        context.shutdown().await.expect("shutdown should succeed");

        assert_eq!(
            registry.states(),
            vec![
                NodeState::Starting,
                NodeState::Ready,
                NodeState::Draining,
                NodeState::Offline,
            ]
        );
        assert_eq!(context.lifecycle().state(), crate::LifecycleState::Stopped);
        assert_eq!(context.health().state(), crate::HealthState::Stopped);
        assert!(context
            .node_registry()
            .get(context.node_id())
            .await
            .expect("node lookup should succeed")
            .is_none());
    }
}

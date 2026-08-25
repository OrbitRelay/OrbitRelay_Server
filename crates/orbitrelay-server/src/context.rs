//! Shared dependencies assembled for one server process.

use std::sync::Arc;

use orbitrelay_core::Metadata;
use orbitrelay_node::{Capability, Node, NodeId, NodeRegistry, NodeState};
use orbitrelay_runtime::Runtime;
use orbitrelay_storage::EventStore;
use orbitrelay_sync::EventBus;

use crate::{HealthStatus, LifecycleState, ServerError, ServerLifecycle};

/// Read-only access to the dependencies of one OrbitRelay server process.
#[derive(Clone)]
pub struct ServerContext {
    local_node: Node,
    lifecycle: ServerLifecycle,
    runtime: Arc<Runtime>,
    event_store: Arc<dyn EventStore>,
    event_bus: Arc<dyn EventBus>,
    node_registry: Arc<dyn NodeRegistry>,
}

impl ServerContext {
    /// Creates a context from already-composed dependencies.
    #[must_use]
    pub fn new(
        node_id: NodeId,
        runtime: Arc<Runtime>,
        event_store: Arc<dyn EventStore>,
        event_bus: Arc<dyn EventBus>,
        node_registry: Arc<dyn NodeRegistry>,
    ) -> Self {
        let local_node = Node::new(
            node_id,
            Metadata::new(),
            NodeState::Ready,
            std::iter::empty::<Capability>(),
        );
        let lifecycle = ServerLifecycle::from_state(LifecycleState::Ready);

        Self::new_composed(
            local_node,
            lifecycle,
            runtime,
            event_store,
            event_bus,
            node_registry,
        )
    }

    pub(crate) fn new_composed(
        local_node: Node,
        lifecycle: ServerLifecycle,
        runtime: Arc<Runtime>,
        event_store: Arc<dyn EventStore>,
        event_bus: Arc<dyn EventBus>,
        node_registry: Arc<dyn NodeRegistry>,
    ) -> Self {
        Self {
            local_node,
            lifecycle,
            runtime,
            event_store,
            event_bus,
            node_registry,
        }
    }

    /// Returns the local node identifier.
    #[must_use]
    pub const fn node_id(&self) -> &NodeId {
        self.local_node.id()
    }

    /// Returns the process lifecycle state machine.
    #[must_use]
    pub const fn lifecycle(&self) -> &ServerLifecycle {
        &self.lifecycle
    }

    /// Returns the read-only process health view.
    #[must_use]
    pub const fn health(&self) -> &HealthStatus {
        self.lifecycle.health()
    }

    /// Returns the composed runtime.
    #[must_use]
    pub fn runtime(&self) -> &Runtime {
        self.runtime.as_ref()
    }

    /// Returns the abstract event store.
    #[must_use]
    pub fn event_store(&self) -> &dyn EventStore {
        self.event_store.as_ref()
    }

    /// Returns the abstract event bus.
    #[must_use]
    pub fn event_bus(&self) -> &dyn EventBus {
        self.event_bus.as_ref()
    }

    /// Returns the abstract node registry.
    #[must_use]
    pub fn node_registry(&self) -> &dyn NodeRegistry {
        self.node_registry.as_ref()
    }

    /// Gracefully drains, unregisters, and stops this server context.
    pub async fn shutdown(&self) -> Result<(), ServerError> {
        self.lifecycle.begin_shutdown()?;
        self.register_node_state(NodeState::Draining).await?;
        self.register_node_state(NodeState::Offline).await?;
        self.node_registry
            .unregister(self.node_id())
            .await
            .map_err(|_| ServerError::node_lifecycle("node unregister failed"))?;
        self.lifecycle.stop()?;
        Ok(())
    }

    async fn register_node_state(&self, state: NodeState) -> Result<(), ServerError> {
        self.node_registry
            .register(Node::new(
                self.local_node.id().clone(),
                self.local_node.metadata().clone(),
                state,
                self.local_node.capabilities().iter().cloned(),
            ))
            .await
            .map_err(|_| ServerError::node_lifecycle("node state registration failed"))
    }
}

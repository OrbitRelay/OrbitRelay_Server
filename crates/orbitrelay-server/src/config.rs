//! Process-level configuration for the composition root.

use orbitrelay_core::Metadata;
use orbitrelay_node::NodeId;

use crate::ServerError;

/// Default capacity for each in-memory event subscription queue.
pub const DEFAULT_SUBSCRIPTION_QUEUE_CAPACITY: usize = 64;

/// Minimal configuration needed to compose a local OrbitRelay server.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerConfig {
    node_id: Option<NodeId>,
    node_metadata: Metadata,
    subscription_queue_capacity: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            node_id: None,
            node_metadata: Metadata::new(),
            subscription_queue_capacity: DEFAULT_SUBSCRIPTION_QUEUE_CAPACITY,
        }
    }
}

impl ServerConfig {
    /// Loads supported environment overrides over the default configuration.
    ///
    /// Supported variables are `ORBITRELAY_NODE_ID` and
    /// `ORBITRELAY_SUBSCRIPTION_QUEUE_CAPACITY`. Unknown variables are ignored.
    pub fn load() -> Result<Self, ServerError> {
        let mut config = Self::default();

        if let Some(value) = std::env::var_os("ORBITRELAY_NODE_ID") {
            let value = value
                .into_string()
                .map_err(|_| ServerError::config("ORBITRELAY_NODE_ID is not valid UTF-8"))?;
            config.node_id =
                Some(value.parse().map_err(|_| {
                    ServerError::config("ORBITRELAY_NODE_ID is not a valid node ID")
                })?);
        }

        if let Some(value) = std::env::var_os("ORBITRELAY_SUBSCRIPTION_QUEUE_CAPACITY") {
            let value = value.into_string().map_err(|_| {
                ServerError::config("ORBITRELAY_SUBSCRIPTION_QUEUE_CAPACITY is not valid UTF-8")
            })?;
            config.subscription_queue_capacity = value.parse().map_err(|_| {
                ServerError::config(
                    "ORBITRELAY_SUBSCRIPTION_QUEUE_CAPACITY must be a positive integer",
                )
            })?;
        }

        config.validate()?;
        Ok(config)
    }

    /// Validates configuration invariants independent of a concrete backend.
    pub fn validate(&self) -> Result<(), ServerError> {
        if self.subscription_queue_capacity == 0 {
            return Err(ServerError::config(
                "subscription queue capacity must be greater than zero",
            ));
        }

        Ok(())
    }

    /// Sets the stable identifier advertised by this process.
    #[must_use]
    pub fn with_node_id(mut self, node_id: NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    /// Sets the business-neutral metadata advertised by this process.
    #[must_use]
    pub fn with_node_metadata(mut self, node_metadata: Metadata) -> Self {
        self.node_metadata = node_metadata;
        self
    }

    /// Sets the bounded capacity of each in-memory subscription queue.
    #[must_use]
    pub const fn with_subscription_queue_capacity(mut self, capacity: usize) -> Self {
        self.subscription_queue_capacity = capacity;
        self
    }

    /// Returns the configured stable node identifier, if one was supplied.
    #[must_use]
    pub const fn node_id(&self) -> Option<&NodeId> {
        self.node_id.as_ref()
    }

    /// Returns the metadata advertised by the local node.
    #[must_use]
    pub const fn node_metadata(&self) -> &Metadata {
        &self.node_metadata
    }

    /// Returns the in-memory subscription queue capacity.
    #[must_use]
    pub const fn subscription_queue_capacity(&self) -> usize {
        self.subscription_queue_capacity
    }
}

#[cfg(test)]
mod tests {
    use super::ServerConfig;
    use crate::ServerError;

    #[test]
    fn default_configuration_is_valid() {
        let config = ServerConfig::default();

        config.validate().expect("default config should be valid");
        assert!(config.node_id().is_none());
        assert_eq!(config.subscription_queue_capacity(), 64);
    }

    #[test]
    fn rejects_zero_queue_capacity() {
        let error = ServerConfig::default()
            .with_subscription_queue_capacity(0)
            .validate()
            .expect_err("zero capacity should be rejected");

        assert!(matches!(error, ServerError::Config { .. }));
    }
}

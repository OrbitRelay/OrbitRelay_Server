//! Subscription identities and asynchronous event consumption.

use std::fmt;

use async_trait::async_trait;
use orbitrelay_core::{CoreError, EntityId};
use orbitrelay_protocol::Event;

use crate::{EventFilter, SyncError};

/// Identifies a subscription within the synchronization layer.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubscriptionId(EntityId);

impl SubscriptionId {
    /// Creates a new random subscription identifier.
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

    /// Parses a subscription identifier from a UUID string.
    pub fn parse(value: &str) -> Result<Self, CoreError> {
        Ok(Self(EntityId::parse(value)?))
    }
}

impl fmt::Display for SubscriptionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A single-consumer asynchronous event subscription.
#[async_trait]
pub trait Subscription: Send {
    /// Returns the subscription identifier.
    fn id(&self) -> &SubscriptionId;

    /// Returns the filter applied to this subscription.
    fn filter(&self) -> &EventFilter;

    /// Waits for the next matching event.
    ///
    /// `None` indicates that the underlying event bus ended normally.
    async fn next_event(&mut self) -> Result<Option<Event>, SyncError>;

    /// Closes the subscription and releases its bus registration.
    async fn close(&mut self) -> Result<(), SyncError>;
}

#[cfg(test)]
mod tests {
    use super::SubscriptionId;

    #[test]
    fn subscription_id_parses_and_displays() {
        let source = "550e8400-e29b-41d4-a716-446655440000";
        let id = SubscriptionId::parse(source).expect("valid UUID should parse");

        assert_eq!(id.to_string(), source);
    }
}

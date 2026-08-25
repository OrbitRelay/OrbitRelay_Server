//! Event publication and subscription abstraction.

use async_trait::async_trait;
use orbitrelay_protocol::Event;

use crate::{EventFilter, Subscription, SyncError};

/// Publishes events and creates filtered subscriptions.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publishes an event to every matching subscription.
    async fn publish(&self, event: Event) -> Result<(), SyncError>;

    /// Creates an independent subscription for a validated filter.
    async fn subscribe(&self, filter: EventFilter) -> Result<Box<dyn Subscription>, SyncError>;
}

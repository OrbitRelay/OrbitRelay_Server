//! Append-only EventStore abstraction.

use async_trait::async_trait;
use orbitrelay_protocol::{Event, EventId};

use crate::{EventPage, EventQuery, StorageError, StoredEvent};

/// Persists immutable protocol events and queries them in append order.
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Appends an event or returns its existing identical record.
    async fn append(&self, event: Event) -> Result<StoredEvent, StorageError>;

    /// Gets one stored event by its protocol EventId.
    async fn get(&self, event_id: &EventId) -> Result<Option<StoredEvent>, StorageError>;

    /// Queries a validated, append-ordered page of stored events.
    async fn query(&self, query: EventQuery) -> Result<EventPage, StorageError>;
}

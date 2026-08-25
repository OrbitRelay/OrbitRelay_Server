//! Event dispatch boundary used after successful action handling.

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_trait::async_trait;
use orbitrelay_protocol::Event;

use crate::PipelineError;

/// Dispatches generated events to downstream infrastructure.
#[async_trait]
pub trait EventPipeline: Send + Sync {
    /// Dispatches one action's generated events as a batch.
    async fn dispatch(&self, events: &[Event]) -> Result<(), PipelineError>;
}

/// An in-memory event pipeline intended for tests and local fixtures.
#[derive(Default)]
pub struct MemoryEventPipeline {
    events: RwLock<Vec<Event>>,
}

impl MemoryEventPipeline {
    /// Creates an empty in-memory event pipeline.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a snapshot of all events received by the pipeline.
    #[must_use]
    pub fn events(&self) -> Vec<Event> {
        self.read_events().clone()
    }

    /// Removes all captured events.
    pub fn clear(&self) {
        self.write_events().clear();
    }

    fn read_events(&self) -> RwLockReadGuard<'_, Vec<Event>> {
        self.events
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write_events(&self) -> RwLockWriteGuard<'_, Vec<Event>> {
        self.events
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[async_trait]
impl EventPipeline for MemoryEventPipeline {
    async fn dispatch(&self, events: &[Event]) -> Result<(), PipelineError> {
        self.write_events().extend_from_slice(events);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::{Metadata, Timestamp};
    use orbitrelay_protocol::{ActionId, ActorId, Event, EventId, EventType, Payload, SessionId};

    use super::{EventPipeline, MemoryEventPipeline};

    #[tokio::test]
    async fn captures_dispatched_events() {
        let pipeline = MemoryEventPipeline::new();
        let event = Event::new(
            EventId::new(),
            SessionId::new(),
            ActorId::new(),
            ActionId::new(),
            EventType::new("test.completed"),
            Timestamp::from_unix_timestamp(1_700_000_000).expect("timestamp is valid"),
            Payload::new(),
            Metadata::new(),
        );

        pipeline
            .dispatch(std::slice::from_ref(&event))
            .await
            .expect("memory pipeline should accept events");

        assert_eq!(pipeline.events(), vec![event]);
        pipeline.clear();
        assert!(pipeline.events().is_empty());
    }
}

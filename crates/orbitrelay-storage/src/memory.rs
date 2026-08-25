//! Thread-safe in-memory EventStore implementation.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use async_trait::async_trait;
use orbitrelay_core::EntityId;
use orbitrelay_protocol::{Event, EventId};

use crate::{EventCursor, EventPage, EventQuery, EventStore, StorageError, StoredEvent};

struct State {
    store_id: EntityId,
    records: Vec<StoredEvent>,
    event_indices: HashMap<EventId, usize>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            store_id: EntityId::new(),
            records: Vec::new(),
            event_indices: HashMap::new(),
        }
    }
}

/// A cloneable, thread-safe, append-only in-memory event store.
#[derive(Clone, Default)]
pub struct MemoryEventStore {
    state: Arc<RwLock<State>>,
}

impl MemoryEventStore {
    /// Creates an empty in-memory event store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn read_state(&self) -> RwLockReadGuard<'_, State> {
        self.state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write_state(&self) -> RwLockWriteGuard<'_, State> {
        self.state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[async_trait]
impl EventStore for MemoryEventStore {
    async fn append(&self, event: Event) -> Result<StoredEvent, StorageError> {
        let mut state = self.write_state();

        if let Some(index) = state.event_indices.get(event.id()).copied() {
            let existing = &state.records[index];
            if existing.event() == &event {
                return Ok(existing.clone());
            }

            return Err(StorageError::EventConflict {
                event_id: event.id().clone(),
            });
        }

        let cursor = EventCursor::for_memory(&state.store_id, state.records.len() + 1);
        let record = StoredEvent::new(cursor, event);
        let index = state.records.len();
        state
            .event_indices
            .insert(record.event().id().clone(), index);
        state.records.push(record.clone());

        Ok(record)
    }

    async fn get(&self, event_id: &EventId) -> Result<Option<StoredEvent>, StorageError> {
        let state = self.read_state();
        Ok(state
            .event_indices
            .get(event_id)
            .map(|index| state.records[*index].clone()))
    }

    async fn query(&self, query: EventQuery) -> Result<EventPage, StorageError> {
        query.validate()?;
        let state = self.read_state();
        let start = match query.after_cursor() {
            Some(cursor) => cursor.memory_position(&state.store_id, state.records.len())?,
            None => 0,
        };

        let mut records = state
            .records
            .iter()
            .skip(start)
            .filter(|record| query.matches(record.event()))
            .take(query.limit() + 1)
            .cloned()
            .collect::<Vec<_>>();

        let next_cursor = if records.len() > query.limit() {
            records.truncate(query.limit());
            records.last().map(|record| record.cursor().clone())
        } else {
            None
        };

        Ok(EventPage::new(records, next_cursor))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use orbitrelay_core::{Metadata, Timestamp};
    use orbitrelay_protocol::{ActionId, ActorId, Event, EventId, EventType, Payload, SessionId};

    use super::MemoryEventStore;
    use crate::{EventCursor, EventQuery, EventStore, StorageError};

    fn event(
        event_id: EventId,
        session_id: SessionId,
        actor_id: ActorId,
        event_type: &str,
        occurred_at: i64,
    ) -> Event {
        Event::new(
            event_id,
            session_id,
            actor_id,
            ActionId::new(),
            EventType::new(event_type),
            Timestamp::from_unix_timestamp(occurred_at).expect("timestamp is valid"),
            Payload::new(),
            Metadata::new(),
        )
    }

    #[tokio::test]
    async fn appends_and_gets_event_by_id() {
        let store = MemoryEventStore::new();
        let event = event(
            EventId::new(),
            SessionId::new(),
            ActorId::new(),
            "document.written",
            100,
        );

        let stored = store
            .append(event.clone())
            .await
            .expect("append should succeed");
        let fetched = store
            .get(event.id())
            .await
            .expect("get should succeed")
            .expect("event should exist");

        assert_eq!(stored, fetched);
        assert_eq!(fetched.event(), &event);
    }

    #[tokio::test]
    async fn repeated_identical_event_is_idempotent() {
        let store = MemoryEventStore::new();
        let event = event(
            EventId::new(),
            SessionId::new(),
            ActorId::new(),
            "document.written",
            100,
        );

        let first = store
            .append(event.clone())
            .await
            .expect("first append should succeed");
        let second = store
            .append(event)
            .await
            .expect("identical append should succeed");
        let page = store
            .query(EventQuery::all())
            .await
            .expect("query should succeed");

        assert_eq!(first, second);
        assert_eq!(page.len(), 1);
    }

    #[tokio::test]
    async fn rejects_conflicting_event_content() {
        let store = MemoryEventStore::new();
        let event_id = EventId::new();
        let session_id = SessionId::new();
        let actor_id = ActorId::new();
        store
            .append(event(
                event_id.clone(),
                session_id.clone(),
                actor_id.clone(),
                "document.written",
                100,
            ))
            .await
            .expect("first append should succeed");

        let error = store
            .append(event(
                event_id.clone(),
                session_id,
                actor_id,
                "document.deleted",
                100,
            ))
            .await
            .expect_err("different content must conflict");

        assert_eq!(error, StorageError::EventConflict { event_id });
    }

    #[tokio::test]
    async fn filters_by_session_and_event_type() {
        let store = MemoryEventStore::new();
        let selected_session = SessionId::new();
        let other_session = SessionId::new();
        let actor_id = ActorId::new();
        store
            .append(event(
                EventId::new(),
                selected_session.clone(),
                actor_id.clone(),
                "document.written",
                100,
            ))
            .await
            .expect("append should succeed");
        store
            .append(event(
                EventId::new(),
                selected_session.clone(),
                actor_id.clone(),
                "document.opened",
                101,
            ))
            .await
            .expect("append should succeed");
        store
            .append(event(
                EventId::new(),
                other_session,
                actor_id,
                "document.written",
                102,
            ))
            .await
            .expect("append should succeed");

        let page = store
            .query(
                EventQuery::for_session(selected_session)
                    .with_event_type(EventType::new("document.written")),
            )
            .await
            .expect("query should succeed");

        assert_eq!(page.len(), 1);
        assert_eq!(
            page.events()[0].event().event_type().as_str(),
            "document.written"
        );
    }

    #[tokio::test]
    async fn paginates_by_append_cursor_and_limit() {
        let store = MemoryEventStore::new();
        let session_id = SessionId::new();
        for occurred_at in 100..103 {
            store
                .append(event(
                    EventId::new(),
                    session_id.clone(),
                    ActorId::new(),
                    "event.recorded",
                    occurred_at,
                ))
                .await
                .expect("append should succeed");
        }

        let first = store
            .query(EventQuery::all().with_limit(2))
            .await
            .expect("first page should succeed");
        let cursor = first
            .next_cursor()
            .cloned()
            .expect("more records should be available");
        let second = store
            .query(EventQuery::all().with_limit(2).after(cursor))
            .await
            .expect("second page should succeed");

        assert_eq!(first.len(), 2);
        assert_eq!(second.len(), 1);
        assert!(second.next_cursor().is_none());
        assert_eq!(
            second.events()[0].event().occurred_at().unix_timestamp(),
            102
        );
    }

    #[tokio::test]
    async fn filters_half_open_time_range() {
        let store = MemoryEventStore::new();
        for occurred_at in [100, 200, 300] {
            store
                .append(event(
                    EventId::new(),
                    SessionId::new(),
                    ActorId::new(),
                    "event.recorded",
                    occurred_at,
                ))
                .await
                .expect("append should succeed");
        }

        let page = store
            .query(EventQuery::all().with_time_range(
                Timestamp::from_unix_timestamp(150).expect("timestamp is valid"),
                Timestamp::from_unix_timestamp(300).expect("timestamp is valid"),
            ))
            .await
            .expect("query should succeed");

        assert_eq!(page.len(), 1);
        assert_eq!(page.events()[0].event().occurred_at().unix_timestamp(), 200);
    }

    #[tokio::test]
    async fn rejects_cursor_from_another_store() {
        let first_store = MemoryEventStore::new();
        let second_store = MemoryEventStore::new();
        let record = first_store
            .append(event(
                EventId::new(),
                SessionId::new(),
                ActorId::new(),
                "event.recorded",
                100,
            ))
            .await
            .expect("append should succeed");
        second_store
            .append(event(
                EventId::new(),
                SessionId::new(),
                ActorId::new(),
                "event.recorded",
                100,
            ))
            .await
            .expect("append should succeed");

        let error = second_store
            .query(EventQuery::all().after(record.cursor().clone()))
            .await
            .expect_err("foreign cursor must be rejected");

        assert!(matches!(error, StorageError::InvalidCursor { .. }));
    }

    #[tokio::test]
    async fn rejects_malformed_cursor() {
        let store = MemoryEventStore::new();
        let error = store
            .query(EventQuery::all().after(EventCursor::from_test_token("not-a-cursor")))
            .await
            .expect_err("malformed cursor must be rejected");

        assert!(matches!(error, StorageError::InvalidCursor { .. }));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn appends_safely_from_multiple_tasks() {
        let store = Arc::new(MemoryEventStore::new());
        let mut tasks = Vec::new();

        for occurred_at in 0..32 {
            let store = store.clone();
            tasks.push(tokio::spawn(async move {
                store
                    .append(event(
                        EventId::new(),
                        SessionId::new(),
                        ActorId::new(),
                        "concurrent.event",
                        occurred_at,
                    ))
                    .await
            }));
        }

        for task in tasks {
            task.await
                .expect("append task should complete")
                .expect("append should succeed");
        }

        let page = store
            .query(EventQuery::all())
            .await
            .expect("query should succeed");
        assert_eq!(page.len(), 32);
    }
}

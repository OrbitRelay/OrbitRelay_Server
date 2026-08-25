//! Stored event records and cursor-based query pages.

use std::fmt;

use orbitrelay_core::EntityId;
use orbitrelay_protocol::Event;
use serde::{Deserialize, Serialize};

use crate::StorageError;

/// An opaque, backend-owned position in append order.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventCursor(String);

impl fmt::Debug for EventCursor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EventCursor(..)")
    }
}

impl EventCursor {
    pub(crate) fn for_memory(store_id: &EntityId, position: usize) -> Self {
        Self(format!("memory:{store_id}:{position}"))
    }

    pub(crate) fn memory_position(
        &self,
        store_id: &EntityId,
        record_count: usize,
    ) -> Result<usize, StorageError> {
        let mut parts = self.0.split(':');
        let backend = parts.next();
        let cursor_store_id = parts.next();
        let position = parts.next();

        if backend != Some("memory")
            || cursor_store_id != Some(store_id.to_string().as_str())
            || parts.next().is_some()
        {
            return Err(StorageError::InvalidCursor {
                reason: "cursor does not belong to this memory event store".to_owned(),
            });
        }

        let position = position
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|position| *position > 0 && *position <= record_count)
            .ok_or_else(|| StorageError::InvalidCursor {
                reason: "cursor position is outside the stored event range".to_owned(),
            })?;

        Ok(position)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn from_test_token(token: impl Into<String>) -> Self {
        Self(token.into())
    }
}

/// An immutable Event paired with its backend append cursor.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StoredEvent {
    cursor: EventCursor,
    event: Event,
}

impl StoredEvent {
    pub(crate) const fn new(cursor: EventCursor, event: Event) -> Self {
        Self { cursor, event }
    }

    /// Returns the opaque append cursor for this record.
    #[must_use]
    pub const fn cursor(&self) -> &EventCursor {
        &self.cursor
    }

    /// Returns the stored protocol event.
    #[must_use]
    pub const fn event(&self) -> &Event {
        &self.event
    }

    /// Consumes the record and returns its protocol event.
    #[must_use]
    pub fn into_event(self) -> Event {
        self.event
    }
}

/// One append-ordered page of stored events.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EventPage {
    events: Vec<StoredEvent>,
    next_cursor: Option<EventCursor>,
}

impl EventPage {
    pub(crate) const fn new(events: Vec<StoredEvent>, next_cursor: Option<EventCursor>) -> Self {
        Self {
            events,
            next_cursor,
        }
    }

    /// Returns the stored events in append order.
    #[must_use]
    pub fn events(&self) -> &[StoredEvent] {
        &self.events
    }

    /// Returns the cursor for requesting the next page, when more matches exist.
    #[must_use]
    pub const fn next_cursor(&self) -> Option<&EventCursor> {
        self.next_cursor.as_ref()
    }

    /// Returns the number of records in this page.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns whether this page contains no records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Consumes the page and returns its stored records.
    #[must_use]
    pub fn into_events(self) -> Vec<StoredEvent> {
        self.events
    }
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::EntityId;

    use super::EventCursor;

    #[test]
    fn cursor_round_trips_without_debugging_its_token() {
        let cursor = EventCursor::for_memory(&EntityId::new(), 7);
        let encoded = serde_json::to_string(&cursor).expect("cursor should serialize");
        let decoded: EventCursor =
            serde_json::from_str(&encoded).expect("cursor should deserialize");

        assert_eq!(decoded, cursor);
        assert_eq!(format!("{cursor:?}"), "EventCursor(..)");
    }
}

//! Structured, backend-portable event queries.

use std::collections::BTreeSet;

use orbitrelay_core::Timestamp;
use orbitrelay_protocol::{ActorId, Event, EventType, SessionId};

use crate::{EventCursor, StorageError};

/// Conditions and pagination controls for append-ordered event queries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventQuery {
    session_id: Option<SessionId>,
    actor_id: Option<ActorId>,
    event_types: BTreeSet<EventType>,
    occurred_from: Option<Timestamp>,
    occurred_until: Option<Timestamp>,
    after: Option<EventCursor>,
    limit: usize,
}

impl EventQuery {
    /// The number of records returned when no explicit limit is supplied.
    pub const DEFAULT_LIMIT: usize = 100;

    /// The largest page that a store accepts through this abstraction.
    pub const MAX_LIMIT: usize = 1_000;

    /// Creates a query accepting events from all sessions and actors.
    #[must_use]
    pub fn all() -> Self {
        Self {
            session_id: None,
            actor_id: None,
            event_types: BTreeSet::new(),
            occurred_from: None,
            occurred_until: None,
            after: None,
            limit: Self::DEFAULT_LIMIT,
        }
    }

    /// Creates a query restricted to one session.
    #[must_use]
    pub fn for_session(session_id: SessionId) -> Self {
        Self {
            session_id: Some(session_id),
            ..Self::all()
        }
    }

    /// Restricts the query to one actor.
    #[must_use]
    pub fn with_actor_id(mut self, actor_id: ActorId) -> Self {
        self.actor_id = Some(actor_id);
        self
    }

    /// Adds an exact event type to the accepted set.
    #[must_use]
    pub fn with_event_type(mut self, event_type: EventType) -> Self {
        self.event_types.insert(event_type);
        self
    }

    /// Restricts occurrence time to the half-open interval `[from, until)`.
    #[must_use]
    pub fn with_time_range(mut self, from: Timestamp, until: Timestamp) -> Self {
        self.occurred_from = Some(from);
        self.occurred_until = Some(until);
        self
    }

    /// Continues the query strictly after an opaque append cursor.
    #[must_use]
    pub fn after(mut self, cursor: EventCursor) -> Self {
        self.after = Some(cursor);
        self
    }

    /// Sets the maximum number of records in the returned page.
    #[must_use]
    pub const fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Returns the selected session, if any.
    #[must_use]
    pub const fn session_id(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }

    /// Returns the selected actor, if any.
    #[must_use]
    pub const fn actor_id(&self) -> Option<&ActorId> {
        self.actor_id.as_ref()
    }

    /// Returns the exact event type set; an empty set accepts all types.
    #[must_use]
    pub const fn event_types(&self) -> &BTreeSet<EventType> {
        &self.event_types
    }

    /// Returns the inclusive lower occurrence-time bound.
    #[must_use]
    pub const fn occurred_from(&self) -> Option<&Timestamp> {
        self.occurred_from.as_ref()
    }

    /// Returns the exclusive upper occurrence-time bound.
    #[must_use]
    pub const fn occurred_until(&self) -> Option<&Timestamp> {
        self.occurred_until.as_ref()
    }

    /// Returns the append cursor after which results begin.
    #[must_use]
    pub const fn after_cursor(&self) -> Option<&EventCursor> {
        self.after.as_ref()
    }

    /// Returns the maximum page size.
    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }

    /// Validates all backend-independent query invariants.
    pub fn validate(&self) -> Result<(), StorageError> {
        if self.limit == 0 || self.limit > Self::MAX_LIMIT {
            return Err(StorageError::InvalidQuery {
                reason: format!("limit must be between 1 and {}", Self::MAX_LIMIT),
            });
        }

        if self
            .event_types
            .iter()
            .any(|event_type| event_type.as_str().trim().is_empty())
        {
            return Err(StorageError::InvalidQuery {
                reason: "event type cannot be empty".to_owned(),
            });
        }

        if self
            .occurred_from
            .as_ref()
            .zip(self.occurred_until.as_ref())
            .is_some_and(|(from, until)| from >= until)
        {
            return Err(StorageError::InvalidQuery {
                reason: "occurred_from must be earlier than occurred_until".to_owned(),
            });
        }

        if self.after.as_ref().is_some_and(EventCursor::is_empty) {
            return Err(StorageError::InvalidCursor {
                reason: "cursor cannot be empty".to_owned(),
            });
        }

        Ok(())
    }

    pub(crate) fn matches(&self, event: &Event) -> bool {
        self.session_id
            .as_ref()
            .is_none_or(|session_id| session_id == event.session_id())
            && self
                .actor_id
                .as_ref()
                .is_none_or(|actor_id| actor_id == event.actor_id())
            && (self.event_types.is_empty() || self.event_types.contains(event.event_type()))
            && self
                .occurred_from
                .as_ref()
                .is_none_or(|from| event.occurred_at() >= from)
            && self
                .occurred_until
                .as_ref()
                .is_none_or(|until| event.occurred_at() < until)
    }
}

impl Default for EventQuery {
    fn default() -> Self {
        Self::all()
    }
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::Timestamp;

    use super::EventQuery;
    use crate::StorageError;

    #[test]
    fn rejects_invalid_limit() {
        let error = EventQuery::all()
            .with_limit(0)
            .validate()
            .expect_err("zero limit should be rejected");

        assert!(matches!(error, StorageError::InvalidQuery { .. }));
    }

    #[test]
    fn rejects_inverted_time_range() {
        let from = Timestamp::from_unix_timestamp(200).expect("timestamp is valid");
        let until = Timestamp::from_unix_timestamp(100).expect("timestamp is valid");
        let error = EventQuery::all()
            .with_time_range(from, until)
            .validate()
            .expect_err("inverted range should be rejected");

        assert!(matches!(error, StorageError::InvalidQuery { .. }));
    }
}

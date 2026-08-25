//! Portable event filters shared by event bus implementations.

use std::collections::BTreeSet;

use orbitrelay_protocol::{ActorId, Event, EventType, SessionId};

use crate::SyncError;

/// A structured event filter using exact protocol identifiers and event types.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventFilter {
    session_id: Option<SessionId>,
    event_types: BTreeSet<EventType>,
    actor_id: Option<ActorId>,
}

impl EventFilter {
    /// Creates a filter that matches every event.
    #[must_use]
    pub fn all() -> Self {
        Self::default()
    }

    /// Creates a filter that matches events from one session.
    #[must_use]
    pub fn for_session(session_id: SessionId) -> Self {
        Self {
            session_id: Some(session_id),
            ..Self::default()
        }
    }

    /// Adds an exact event type to the accepted set.
    #[must_use]
    pub fn with_event_type(mut self, event_type: EventType) -> Self {
        self.event_types.insert(event_type);
        self
    }

    /// Restricts the filter to events originating from one actor.
    #[must_use]
    pub fn with_actor_id(mut self, actor_id: ActorId) -> Self {
        self.actor_id = Some(actor_id);
        self
    }

    /// Returns the selected session, or `None` when all sessions are accepted.
    #[must_use]
    pub const fn session_id(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }

    /// Returns the exact accepted event types.
    #[must_use]
    pub const fn event_types(&self) -> &BTreeSet<EventType> {
        &self.event_types
    }

    /// Returns the selected actor, or `None` when all actors are accepted.
    #[must_use]
    pub const fn actor_id(&self) -> Option<&ActorId> {
        self.actor_id.as_ref()
    }

    /// Returns whether an event satisfies every configured condition.
    #[must_use]
    pub fn matches(&self, event: &Event) -> bool {
        self.session_id
            .as_ref()
            .is_none_or(|session_id| session_id == event.session_id())
            && (self.event_types.is_empty() || self.event_types.contains(event.event_type()))
            && self
                .actor_id
                .as_ref()
                .is_none_or(|actor_id| actor_id == event.actor_id())
    }

    pub(crate) fn validate(&self) -> Result<(), SyncError> {
        if self
            .event_types
            .iter()
            .any(|event_type| event_type.as_str().trim().is_empty())
        {
            return Err(SyncError::InvalidFilter {
                reason: "event type cannot be empty".to_owned(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::{Metadata, Timestamp};
    use orbitrelay_protocol::{ActionId, ActorId, Event, EventId, EventType, Payload, SessionId};

    use super::EventFilter;

    fn event(session_id: SessionId, actor_id: ActorId, event_type: &str) -> Event {
        Event::new(
            EventId::new(),
            session_id,
            actor_id,
            ActionId::new(),
            EventType::new(event_type),
            Timestamp::from_unix_timestamp(1_700_000_000).expect("timestamp is valid"),
            Payload::new(),
            Metadata::new(),
        )
    }

    #[test]
    fn combines_filter_conditions_with_and_semantics() {
        let session_id = SessionId::new();
        let actor_id = ActorId::new();
        let filter = EventFilter::for_session(session_id.clone())
            .with_event_type(EventType::new("canvas.drawn"))
            .with_actor_id(actor_id.clone());

        assert!(filter.matches(&event(session_id.clone(), actor_id.clone(), "canvas.drawn")));
        assert!(!filter.matches(&event(SessionId::new(), actor_id.clone(), "canvas.drawn")));
        assert!(!filter.matches(&event(session_id.clone(), ActorId::new(), "canvas.drawn")));
        assert!(!filter.matches(&event(session_id, actor_id, "canvas.cleared")));
    }

    #[test]
    fn empty_event_type_set_matches_all_types() {
        let filter = EventFilter::all();

        assert!(filter.matches(&event(SessionId::new(), ActorId::new(), "plugin.custom")));
    }
}

//! Immutable facts produced after actions are processed.

use std::fmt;

use orbitrelay_core::{Metadata, Timestamp};
use serde::{Deserialize, Serialize};

use crate::{ActionId, ActorId, EventId, Payload, SessionId};

/// An extensible event type such as `canvas.drawn`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventType(String);

impl EventType {
    /// Creates an event type without imposing business-specific validation.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the event type string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A fact that occurred after an action was processed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    id: EventId,
    session_id: SessionId,
    actor_id: ActorId,
    action_id: ActionId,
    event_type: EventType,
    occurred_at: Timestamp,
    payload: Payload,
    metadata: Metadata,
}

impl Event {
    /// Creates an event fact linked to its originating action.
    #[must_use]
    pub const fn new(
        id: EventId,
        session_id: SessionId,
        actor_id: ActorId,
        action_id: ActionId,
        event_type: EventType,
        occurred_at: Timestamp,
        payload: Payload,
        metadata: Metadata,
    ) -> Self {
        Self {
            id,
            session_id,
            actor_id,
            action_id,
            event_type,
            occurred_at,
            payload,
            metadata,
        }
    }

    /// Returns the event identifier.
    #[must_use]
    pub const fn id(&self) -> &EventId {
        &self.id
    }

    /// Returns the session in which the fact occurred.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns the actor that originated the action.
    #[must_use]
    pub const fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }

    /// Returns the action that caused the event.
    #[must_use]
    pub const fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    /// Returns the extensible event type.
    #[must_use]
    pub const fn event_type(&self) -> &EventType {
        &self.event_type
    }

    /// Returns when the fact occurred.
    #[must_use]
    pub const fn occurred_at(&self) -> &Timestamp {
        &self.occurred_at
    }

    /// Returns the event payload.
    #[must_use]
    pub const fn payload(&self) -> &Payload {
        &self.payload
    }

    /// Returns the event metadata.
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::{Metadata, Timestamp};
    use serde_json::json;

    use super::{Event, EventType};
    use crate::{ActionId, ActorId, EventId, Payload, SessionId};

    #[test]
    fn constructs_and_round_trips_an_event() {
        let action_id = ActionId::new();
        let mut payload = Payload::new();
        payload.insert("revision", json!(3));
        let event = Event::new(
            EventId::new(),
            SessionId::new(),
            ActorId::new(),
            action_id.clone(),
            EventType::new("document.written"),
            Timestamp::from_unix_timestamp(1_700_000_001).expect("timestamp should be valid"),
            payload,
            Metadata::new(),
        );

        assert_eq!(event.action_id(), &action_id);
        assert_eq!(event.event_type().as_str(), "document.written");

        let encoded = serde_json::to_string(&event).expect("event should serialize");
        let decoded: Event = serde_json::from_str(&encoded).expect("event should deserialize");

        assert_eq!(decoded, event);
    }
}

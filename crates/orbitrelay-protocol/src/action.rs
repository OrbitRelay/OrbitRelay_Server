//! Requests issued by protocol actors.

use std::fmt;

use orbitrelay_core::{Metadata, Timestamp};
use serde::{Deserialize, Serialize};

use crate::{ActionId, ActorId, Payload, SessionId};

/// An extensible action type such as `canvas.draw`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionType(String);

impl ActionType {
    /// Creates an action type without imposing business-specific validation.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the action type string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ActionType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A request by an actor to perform an operation in a session.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Action {
    id: ActionId,
    session_id: SessionId,
    actor_id: ActorId,
    action_type: ActionType,
    requested_at: Timestamp,
    payload: Payload,
    metadata: Metadata,
}

impl Action {
    /// Creates an action request.
    #[must_use]
    pub const fn new(
        id: ActionId,
        session_id: SessionId,
        actor_id: ActorId,
        action_type: ActionType,
        requested_at: Timestamp,
        payload: Payload,
        metadata: Metadata,
    ) -> Self {
        Self {
            id,
            session_id,
            actor_id,
            action_type,
            requested_at,
            payload,
            metadata,
        }
    }

    /// Returns the action identifier.
    #[must_use]
    pub const fn id(&self) -> &ActionId {
        &self.id
    }

    /// Returns the target session identifier.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns the requesting actor identifier.
    #[must_use]
    pub const fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }

    /// Returns the extensible action type.
    #[must_use]
    pub const fn action_type(&self) -> &ActionType {
        &self.action_type
    }

    /// Returns when the action was requested.
    #[must_use]
    pub const fn requested_at(&self) -> &Timestamp {
        &self.requested_at
    }

    /// Returns the action payload.
    #[must_use]
    pub const fn payload(&self) -> &Payload {
        &self.payload
    }

    /// Returns the action metadata.
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::{Metadata, Timestamp};
    use serde_json::json;

    use super::{Action, ActionType};
    use crate::{ActionId, ActorId, Payload, SessionId};

    #[test]
    fn constructs_and_round_trips_an_action() {
        let mut payload = Payload::new();
        payload.insert("document_id", json!("document-1"));
        let action = Action::new(
            ActionId::new(),
            SessionId::new(),
            ActorId::new(),
            ActionType::new("document.write"),
            Timestamp::from_unix_timestamp(1_700_000_000).expect("timestamp should be valid"),
            payload,
            Metadata::new(),
        );

        assert_eq!(action.action_type().as_str(), "document.write");
        assert_eq!(
            action.payload().get("document_id"),
            Some(&json!("document-1"))
        );

        let encoded = serde_json::to_string(&action).expect("action should serialize");
        let decoded: Action = serde_json::from_str(&encoded).expect("action should deserialize");

        assert_eq!(decoded, action);
    }
}

//! Versioned envelopes used to carry protocol messages.

use std::fmt;

use orbitrelay_core::Version;
use serde::{Deserialize, Serialize};

use crate::MessageId;

/// An extensible message type such as `action` or `event`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageType(String);

impl MessageType {
    /// Creates a message type without imposing transport-specific validation.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the message type string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A versioned message envelope with a strongly typed payload.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    version: Version,
    message_id: MessageId,
    message_type: MessageType,
    payload: T,
}

impl<T> MessageEnvelope<T> {
    /// Wraps a typed payload in a protocol message envelope.
    #[must_use]
    pub const fn new(
        version: Version,
        message_id: MessageId,
        message_type: MessageType,
        payload: T,
    ) -> Self {
        Self {
            version,
            message_id,
            message_type,
            payload,
        }
    }

    /// Returns the protocol version.
    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }

    /// Returns the message identifier.
    #[must_use]
    pub const fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    /// Returns the extensible message type.
    #[must_use]
    pub const fn message_type(&self) -> &MessageType {
        &self.message_type
    }

    /// Returns the typed message payload.
    #[must_use]
    pub const fn payload(&self) -> &T {
        &self.payload
    }

    /// Consumes the envelope and returns its typed payload.
    #[must_use]
    pub fn into_payload(self) -> T {
        self.payload
    }
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::{Metadata, Timestamp, Version};

    use super::{MessageEnvelope, MessageType};
    use crate::{Action, ActionId, ActionType, ActorId, MessageId, Payload, SessionId};

    #[test]
    fn serializes_and_deserializes_a_typed_envelope() {
        let action = Action::new(
            ActionId::new(),
            SessionId::new(),
            ActorId::new(),
            ActionType::new("canvas.draw"),
            Timestamp::from_unix_timestamp(1_700_000_000).expect("timestamp should be valid"),
            Payload::new(),
            Metadata::new(),
        );
        let envelope = MessageEnvelope::new(
            Version::new(0, 1, 0),
            MessageId::new(),
            MessageType::new("action"),
            action,
        );

        let encoded = serde_json::to_string(&envelope).expect("envelope should serialize");
        let decoded: MessageEnvelope<Action> =
            serde_json::from_str(&encoded).expect("envelope should deserialize");

        assert_eq!(decoded, envelope);
        assert_eq!(decoded.message_type().as_str(), "action");
        assert_eq!(decoded.version(), Version::new(0, 1, 0));
    }
}

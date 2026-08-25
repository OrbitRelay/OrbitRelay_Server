//! Type-safe identifiers for protocol domain objects.

use std::{fmt, str::FromStr};

use orbitrelay_core::{CoreError, EntityId};
use serde::{Deserialize, Serialize};

macro_rules! define_domain_id {
    ($(#[$attribute:meta])* $name:ident) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(EntityId);

        impl $name {
            /// Creates a new random identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(EntityId::new())
            }

            /// Wraps an existing core entity identifier.
            #[must_use]
            pub const fn from_entity_id(value: EntityId) -> Self {
                Self(value)
            }

            /// Returns the wrapped core entity identifier.
            #[must_use]
            pub const fn as_entity_id(&self) -> &EntityId {
                &self.0
            }

            /// Parses an identifier from its UUID string representation.
            pub fn parse(value: &str) -> Result<Self, CoreError> {
                Ok(Self(EntityId::parse(value)?))
            }
        }

        impl FromStr for $name {
            type Err = CoreError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

define_domain_id!(
    /// Identifies an actor participating in the protocol.
    ///
    /// Domain identifiers are intentionally incompatible with each other:
    ///
    /// ```compile_fail
    /// use orbitrelay_protocol::{ActorId, SessionId};
    ///
    /// fn accept_actor_id(_: ActorId) {}
    /// accept_actor_id(SessionId::new());
    /// ```
    ActorId
);

define_domain_id!(
    /// Identifies a real-time collaboration session.
    SessionId
);

define_domain_id!(
    /// Identifies an action request.
    ActionId
);

define_domain_id!(
    /// Identifies an event fact.
    EventId
);

define_domain_id!(
    /// Identifies a protocol message envelope.
    MessageId
);

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use super::{ActionId, ActorId, EventId, MessageId, SessionId};

    #[test]
    fn domain_ids_are_distinct_types() {
        assert_ne!(TypeId::of::<ActorId>(), TypeId::of::<SessionId>());
        assert_ne!(TypeId::of::<ActionId>(), TypeId::of::<EventId>());
        assert_ne!(TypeId::of::<EventId>(), TypeId::of::<MessageId>());
    }

    #[test]
    fn serializes_and_deserializes_transparently() {
        let id = ActorId::new();
        let encoded = serde_json::to_string(&id).expect("actor id should serialize");
        let decoded: ActorId = serde_json::from_str(&encoded).expect("actor id should deserialize");

        assert_eq!(decoded, id);
        assert_eq!(encoded, format!("\"{id}\""));
    }

    #[test]
    fn parses_and_displays_domain_ids() {
        let source = "550e8400-e29b-41d4-a716-446655440000";
        let id: EventId = source.parse().expect("valid UUID should parse");

        assert_eq!(id.to_string(), source);
    }
}

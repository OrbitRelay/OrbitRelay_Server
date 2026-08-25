//! Actors that participate in OrbitRelay protocol interactions.

use orbitrelay_core::Metadata;
use serde::{Deserialize, Serialize};

use crate::ActorId;

/// The infrastructure-level category of an actor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    /// A person interacting with the system.
    Human,
    /// An autonomous or assisted software agent.
    Agent,
    /// An extension that participates through the protocol.
    Plugin,
    /// Another infrastructure service.
    Service,
}

/// A participant that can interact through the OrbitRelay protocol.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Actor {
    id: ActorId,
    actor_type: ActorType,
    metadata: Metadata,
}

impl Actor {
    /// Creates an actor with infrastructure type and business-neutral metadata.
    #[must_use]
    pub const fn new(id: ActorId, actor_type: ActorType, metadata: Metadata) -> Self {
        Self {
            id,
            actor_type,
            metadata,
        }
    }

    /// Returns the actor identifier.
    #[must_use]
    pub const fn id(&self) -> &ActorId {
        &self.id
    }

    /// Returns the infrastructure actor type.
    #[must_use]
    pub const fn actor_type(&self) -> ActorType {
        self.actor_type
    }

    /// Returns the actor metadata.
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::Metadata;

    use super::{Actor, ActorType};
    use crate::ActorId;

    #[test]
    fn creates_and_round_trips_an_agent_actor() {
        let mut metadata = Metadata::new();
        metadata.insert("role", "facilitator");
        let actor = Actor::new(ActorId::new(), ActorType::Agent, metadata);

        assert_eq!(actor.actor_type(), ActorType::Agent);
        assert_eq!(actor.metadata().get("role"), Some("facilitator"));

        let encoded = serde_json::to_string(&actor).expect("actor should serialize");
        let decoded: Actor = serde_json::from_str(&encoded).expect("actor should deserialize");

        assert_eq!(decoded, actor);
        assert!(encoded.contains("\"actor_type\":\"agent\""));
    }
}

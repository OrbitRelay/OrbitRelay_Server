//! Business-neutral real-time collaboration sessions.

use orbitrelay_core::{Metadata, Timestamp};
use serde::{Deserialize, Serialize};

use crate::SessionId;

/// A real-time collaboration space without connection or permission state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Session {
    id: SessionId,
    created_at: Timestamp,
    metadata: Metadata,
}

impl Session {
    /// Creates a collaboration session.
    #[must_use]
    pub const fn new(id: SessionId, created_at: Timestamp, metadata: Metadata) -> Self {
        Self {
            id,
            created_at,
            metadata,
        }
    }

    /// Returns the session identifier.
    #[must_use]
    pub const fn id(&self) -> &SessionId {
        &self.id
    }

    /// Returns when the session was created.
    #[must_use]
    pub const fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    /// Returns the session metadata.
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::{Metadata, Timestamp};

    use super::Session;
    use crate::SessionId;

    #[test]
    fn creates_and_round_trips_a_session() {
        let session = Session::new(
            SessionId::new(),
            Timestamp::from_unix_timestamp(1_700_000_000).expect("timestamp should be valid"),
            Metadata::new(),
        );

        let encoded = serde_json::to_string(&session).expect("session should serialize");
        let decoded: Session = serde_json::from_str(&encoded).expect("session should deserialize");

        assert_eq!(decoded, session);
    }
}

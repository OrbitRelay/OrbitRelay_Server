//! Database-neutral storage errors.

use orbitrelay_protocol::EventId;
use thiserror::Error;

/// Errors produced while appending or querying stored events.
#[derive(Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum StorageError {
    /// A query contains an unsupported or contradictory condition.
    #[error("invalid event query: {reason}")]
    InvalidQuery {
        /// The reason the query was rejected.
        reason: String,
    },

    /// A cursor is malformed, belongs to another store, or is out of range.
    #[error("invalid event cursor: {reason}")]
    InvalidCursor {
        /// The reason the cursor was rejected.
        reason: String,
    },

    /// An EventId is already associated with different event content.
    #[error("event `{event_id}` conflicts with an existing stored event")]
    EventConflict {
        /// The conflicting event identifier.
        event_id: EventId,
    },

    /// A configured storage backend is temporarily unavailable.
    #[error("storage backend unavailable: {message}")]
    BackendUnavailable {
        /// A backend-neutral availability message.
        message: String,
    },

    /// A storage backend failed an operation.
    #[error("storage backend failure: {message}")]
    BackendFailure {
        /// A backend-neutral failure message.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::StorageError;

    #[test]
    fn formats_backend_neutral_errors() {
        let error = StorageError::BackendUnavailable {
            message: "temporarily offline".to_owned(),
        };

        assert_eq!(
            error.to_string(),
            "storage backend unavailable: temporarily offline"
        );
    }
}

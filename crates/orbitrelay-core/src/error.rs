//! Errors produced by foundational OrbitRelay value types.

use thiserror::Error;

/// Errors that can occur while constructing or decoding core values.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CoreError {
    /// The supplied value could not be parsed as an entity identifier.
    #[error("invalid entity id: {0}")]
    InvalidEntityId(#[from] uuid::Error),

    /// The supplied value could not be converted into a valid timestamp.
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(#[from] time::error::ComponentRange),
}

#[cfg(test)]
mod tests {
    use super::CoreError;

    #[test]
    fn constructs_invalid_entity_id_error() {
        let source = uuid::Uuid::parse_str("not-an-entity-id").expect_err("input must be invalid");
        let error = CoreError::from(source);

        assert!(matches!(error, CoreError::InvalidEntityId(_)));
        assert!(error.to_string().starts_with("invalid entity id:"));
    }

    #[test]
    fn constructs_invalid_timestamp_error() {
        let source = time::OffsetDateTime::from_unix_timestamp(i64::MAX)
            .expect_err("timestamp must be outside the supported range");
        let error = CoreError::from(source);

        assert!(matches!(error, CoreError::InvalidTimestamp(_)));
        assert!(error.to_string().starts_with("invalid timestamp:"));
    }
}

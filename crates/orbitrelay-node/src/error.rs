//! Backend-neutral node registration errors.

use thiserror::Error;

/// Errors produced while validating or registering node descriptions.
#[derive(Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum NodeError {
    /// A node description violates a registry-independent invariant.
    #[error("invalid node: {reason}")]
    InvalidNode {
        /// The reason the node description was rejected.
        reason: String,
    },

    /// The configured node registry is temporarily unavailable.
    #[error("node registry unavailable: {message}")]
    RegistryUnavailable {
        /// A backend-neutral availability message.
        message: String,
    },

    /// A node registry failed an operation.
    #[error("node registry failure: {message}")]
    RegistryFailure {
        /// A backend-neutral failure message.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::NodeError;

    #[test]
    fn formats_backend_neutral_errors() {
        let error = NodeError::RegistryUnavailable {
            message: "temporarily offline".to_owned(),
        };

        assert_eq!(
            error.to_string(),
            "node registry unavailable: temporarily offline"
        );
    }
}

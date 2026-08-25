//! Errors produced by event buses and subscriptions.

use thiserror::Error;

use crate::SubscriptionId;

/// Errors produced by real-time event propagation.
#[derive(Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum SyncError {
    /// A subscriber could not keep up with its bounded event queue.
    #[error("subscription `{subscription_id}` lagged and missed {missed} event(s)")]
    SubscriberLagged {
        /// The lagging subscription.
        subscription_id: SubscriptionId,
        /// The number of events dropped since the previous notification.
        missed: u64,
    },

    /// An operation was attempted on an explicitly closed subscription.
    #[error("subscription `{subscription_id}` is closed")]
    SubscriptionClosed {
        /// The closed subscription.
        subscription_id: SubscriptionId,
    },

    /// A filter cannot be represented by the event bus.
    #[error("invalid event filter: {reason}")]
    InvalidFilter {
        /// The reason the filter was rejected.
        reason: String,
    },

    /// A memory event bus was configured with an invalid queue capacity.
    #[error("subscription queue capacity must be greater than zero")]
    InvalidQueueCapacity,
}

#[cfg(test)]
mod tests {
    use super::SyncError;
    use crate::SubscriptionId;

    #[test]
    fn formats_lagged_error() {
        let error = SyncError::SubscriberLagged {
            subscription_id: SubscriptionId::new(),
            missed: 2,
        };

        assert!(error.to_string().contains("missed 2 event(s)"));
    }
}

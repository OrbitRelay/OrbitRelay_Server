//! Explicit dependency injection ports used during action execution.

use std::sync::Arc;

#[cfg(any(test, feature = "test-utils"))]
use std::sync::RwLock;

use async_trait::async_trait;
use orbitrelay_core::Timestamp;
use orbitrelay_protocol::Action;

use crate::AuthorizationError;

/// An external authorization boundary for action execution.
#[async_trait]
pub trait ActionAuthorizer: Send + Sync {
    /// Authorizes an action or returns the reason it was rejected.
    async fn authorize(&self, action: &Action) -> Result<(), AuthorizationError>;
}

/// Supplies UTC timestamps without coupling handlers to the system clock.
pub trait Clock: Send + Sync {
    /// Returns the current UTC timestamp.
    fn now(&self) -> Timestamp;
}

/// A production clock backed by the system's current UTC time.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now_utc()
    }
}

/// Dependencies that are explicitly available during action execution.
#[derive(Clone)]
pub struct RuntimeContext {
    clock: Arc<dyn Clock>,
    authorizer: Arc<dyn ActionAuthorizer>,
}

impl RuntimeContext {
    /// Creates a runtime context from its clock and authorization ports.
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>, authorizer: Arc<dyn ActionAuthorizer>) -> Self {
        Self { clock, authorizer }
    }

    /// Returns the clock available to handlers.
    #[must_use]
    pub fn clock(&self) -> &dyn Clock {
        self.clock.as_ref()
    }

    pub(crate) fn authorizer(&self) -> &dyn ActionAuthorizer {
        self.authorizer.as_ref()
    }
}

/// A deterministic clock intended for tests and local fixtures.
#[cfg(any(test, feature = "test-utils"))]
#[derive(Debug)]
pub struct MockClock {
    current: RwLock<Timestamp>,
}

#[cfg(any(test, feature = "test-utils"))]
impl MockClock {
    /// Creates a clock fixed at the supplied timestamp.
    #[must_use]
    pub fn new(current: Timestamp) -> Self {
        Self {
            current: RwLock::new(current),
        }
    }

    /// Changes the timestamp returned by subsequent calls to [`Clock::now`].
    pub fn set(&self, current: Timestamp) {
        let mut value = self
            .current
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *value = current;
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl Clock for MockClock {
    fn now(&self) -> Timestamp {
        self.current
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

/// A test authorizer that accepts every action without applying policy.
#[cfg(any(test, feature = "test-utils"))]
#[derive(Clone, Copy, Debug, Default)]
pub struct AllowAllAuthorizer;

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl ActionAuthorizer for AllowAllAuthorizer {
    async fn authorize(&self, _action: &Action) -> Result<(), AuthorizationError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::Timestamp;

    use super::{Clock, MockClock};

    #[test]
    fn mock_clock_returns_and_updates_fixed_time() {
        let first = Timestamp::from_unix_timestamp(1_700_000_000).expect("timestamp is valid");
        let second = Timestamp::from_unix_timestamp(1_700_000_001).expect("timestamp is valid");
        let clock = MockClock::new(first.clone());

        assert_eq!(clock.now(), first);
        assert_eq!(clock.now(), first);
        clock.set(second.clone());
        assert_eq!(clock.now(), second);
    }
}

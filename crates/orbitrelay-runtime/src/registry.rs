//! Thread-safe registration and lookup of action handlers.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use orbitrelay_protocol::ActionType;

use crate::{ActionHandler, RegistryError};

/// A thread-safe mapping from action types to shared handlers.
#[derive(Default)]
pub struct HandlerRegistry {
    handlers: RwLock<HashMap<ActionType, Arc<dyn ActionHandler>>>,
}

impl HandlerRegistry {
    /// Creates an empty handler registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a handler without replacing an existing registration.
    pub fn register(
        &self,
        action_type: ActionType,
        handler: Arc<dyn ActionHandler>,
    ) -> Result<(), RegistryError> {
        let mut handlers = self.write_handlers();
        if handlers.contains_key(&action_type) {
            return Err(RegistryError::AlreadyRegistered { action_type });
        }

        handlers.insert(action_type, handler);
        Ok(())
    }

    /// Removes and returns the handler registered for an action type.
    pub fn unregister(&self, action_type: &ActionType) -> Option<Arc<dyn ActionHandler>> {
        self.write_handlers().remove(action_type)
    }

    /// Returns a shared handler for an action type.
    ///
    /// The registry lock is released before the returned handler can be awaited.
    #[must_use]
    pub fn get(&self, action_type: &ActionType) -> Option<Arc<dyn ActionHandler>> {
        self.read_handlers().get(action_type).cloned()
    }

    /// Returns whether an action type has a registered handler.
    #[must_use]
    pub fn contains(&self, action_type: &ActionType) -> bool {
        self.read_handlers().contains_key(action_type)
    }

    fn read_handlers(&self) -> RwLockReadGuard<'_, HashMap<ActionType, Arc<dyn ActionHandler>>> {
        self.handlers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write_handlers(&self) -> RwLockWriteGuard<'_, HashMap<ActionType, Arc<dyn ActionHandler>>> {
        self.handlers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use orbitrelay_protocol::{Action, ActionType};

    use super::HandlerRegistry;
    use crate::{ActionHandler, EventDraft, HandlerError, RegistryError, RuntimeContext};

    struct EmptyHandler;

    #[async_trait]
    impl ActionHandler for EmptyHandler {
        async fn validate(
            &self,
            _action: &Action,
            _context: &RuntimeContext,
        ) -> Result<(), HandlerError> {
            Ok(())
        }

        async fn handle(
            &self,
            _action: &Action,
            _context: &RuntimeContext,
        ) -> Result<Vec<EventDraft>, HandlerError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn registers_gets_and_unregisters_handlers() {
        let registry = HandlerRegistry::new();
        let action_type = ActionType::new("test.action");

        registry
            .register(action_type.clone(), Arc::new(EmptyHandler))
            .expect("first registration should succeed");
        assert!(registry.contains(&action_type));
        assert!(registry.get(&action_type).is_some());
        assert!(registry.unregister(&action_type).is_some());
        assert!(!registry.contains(&action_type));
    }

    #[test]
    fn rejects_duplicate_registration() {
        let registry = HandlerRegistry::new();
        let action_type = ActionType::new("test.action");
        registry
            .register(action_type.clone(), Arc::new(EmptyHandler))
            .expect("first registration should succeed");

        let error = registry
            .register(action_type, Arc::new(EmptyHandler))
            .expect_err("duplicate registration should fail");

        assert!(matches!(error, RegistryError::AlreadyRegistered { .. }));
    }
}

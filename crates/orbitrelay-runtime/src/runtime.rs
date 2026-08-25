//! Action lifecycle orchestration.

use std::sync::Arc;

use orbitrelay_protocol::{Action, Event, EventId};

use crate::{EventPipeline, HandlerRegistry, RuntimeContext, RuntimeError};

/// The protocol execution engine.
pub struct Runtime {
    registry: Arc<HandlerRegistry>,
    context: RuntimeContext,
    pipeline: Arc<dyn EventPipeline>,
}

impl Runtime {
    /// Creates a runtime from its handler registry, context, and event pipeline.
    #[must_use]
    pub fn new(
        registry: Arc<HandlerRegistry>,
        context: RuntimeContext,
        pipeline: Arc<dyn EventPipeline>,
    ) -> Self {
        Self {
            registry,
            context,
            pipeline,
        }
    }

    /// Executes the complete action lifecycle and returns dispatched events.
    pub async fn execute(&self, action: Action) -> Result<Vec<Event>, RuntimeError> {
        let handler = self.registry.get(action.action_type()).ok_or_else(|| {
            RuntimeError::HandlerNotFound {
                action_type: action.action_type().clone(),
            }
        })?;

        handler
            .validate(&action, &self.context)
            .await
            .map_err(|source| RuntimeError::ValidationFailed {
                action_id: action.id().clone(),
                source,
            })?;

        self.context
            .authorizer()
            .authorize(&action)
            .await
            .map_err(|source| RuntimeError::AuthorizationFailed {
                action_id: action.id().clone(),
                source,
            })?;

        let drafts = handler
            .handle(&action, &self.context)
            .await
            .map_err(|source| RuntimeError::HandlerFailed {
                action_id: action.id().clone(),
                source,
            })?;

        let events = drafts
            .into_iter()
            .map(|draft| {
                let (event_type, payload, metadata) = draft.into_parts();
                Event::new(
                    EventId::new(),
                    action.session_id().clone(),
                    action.actor_id().clone(),
                    action.id().clone(),
                    event_type,
                    self.context.clock().now(),
                    payload,
                    metadata,
                )
            })
            .collect::<Vec<_>>();

        self.pipeline
            .dispatch(&events)
            .await
            .map_err(|source| RuntimeError::PipelineFailed {
                action_id: action.id().clone(),
                source,
            })?;

        Ok(events)
    }

    /// Returns the dynamic handler registry.
    #[must_use]
    pub fn registry(&self) -> &HandlerRegistry {
        &self.registry
    }

    /// Returns the runtime dependency context.
    #[must_use]
    pub const fn context(&self) -> &RuntimeContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use async_trait::async_trait;
    use orbitrelay_core::{Metadata, Timestamp};
    use orbitrelay_protocol::{
        Action, ActionId, ActionType, ActorId, EventType, Payload, SessionId,
    };

    use super::Runtime;
    use crate::{
        ActionAuthorizer, ActionHandler, AllowAllAuthorizer, AuthorizationError, EventDraft,
        HandlerError, HandlerRegistry, MemoryEventPipeline, MockClock, RuntimeContext,
        RuntimeError,
    };

    struct TestHandler {
        handled: Arc<AtomicBool>,
        validation_error: Option<&'static str>,
        handler_error: Option<&'static str>,
    }

    #[async_trait]
    impl ActionHandler for TestHandler {
        async fn validate(
            &self,
            _action: &Action,
            _context: &RuntimeContext,
        ) -> Result<(), HandlerError> {
            match self.validation_error {
                Some(message) => Err(HandlerError::new(message)),
                None => Ok(()),
            }
        }

        async fn handle(
            &self,
            _action: &Action,
            _context: &RuntimeContext,
        ) -> Result<Vec<EventDraft>, HandlerError> {
            self.handled.store(true, Ordering::SeqCst);
            if let Some(message) = self.handler_error {
                return Err(HandlerError::new(message));
            }

            Ok(vec![EventDraft::new(
                EventType::new("canvas.drawn"),
                Payload::new(),
                Metadata::new(),
            )])
        }
    }

    struct RejectingAuthorizer;

    #[async_trait]
    impl ActionAuthorizer for RejectingAuthorizer {
        async fn authorize(&self, _action: &Action) -> Result<(), AuthorizationError> {
            Err(AuthorizationError::new("action denied"))
        }
    }

    fn action(action_type: &str, actor_id: ActorId, session_id: SessionId) -> Action {
        Action::new(
            ActionId::new(),
            session_id,
            actor_id,
            ActionType::new(action_type),
            Timestamp::from_unix_timestamp(1_600_000_000).expect("timestamp is valid"),
            Payload::new(),
            Metadata::new(),
        )
    }

    fn handler(handled: Arc<AtomicBool>) -> TestHandler {
        TestHandler {
            handled,
            validation_error: None,
            handler_error: None,
        }
    }

    #[tokio::test]
    async fn finds_handler_and_materializes_complete_event() {
        let registry = Arc::new(HandlerRegistry::new());
        let handled = Arc::new(AtomicBool::new(false));
        registry
            .register(
                ActionType::new("canvas.draw"),
                Arc::new(handler(handled.clone())),
            )
            .expect("handler registration should succeed");

        let occurred_at =
            Timestamp::from_unix_timestamp(1_700_000_000).expect("timestamp is valid");
        let context = RuntimeContext::new(
            Arc::new(MockClock::new(occurred_at.clone())),
            Arc::new(AllowAllAuthorizer),
        );
        let pipeline = Arc::new(MemoryEventPipeline::new());
        let runtime = Runtime::new(registry, context, pipeline.clone());
        let actor_id = ActorId::new();
        let session_id = SessionId::new();
        let action = action("canvas.draw", actor_id.clone(), session_id.clone());
        let action_id = action.id().clone();

        let events = runtime
            .execute(action)
            .await
            .expect("action should execute");

        assert!(handled.load(Ordering::SeqCst));
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.actor_id(), &actor_id);
        assert_eq!(event.session_id(), &session_id);
        assert_eq!(event.action_id(), &action_id);
        assert_eq!(event.occurred_at(), &occurred_at);
        assert_eq!(event.event_type().as_str(), "canvas.drawn");
        assert_eq!(pipeline.events(), events);
    }

    #[tokio::test]
    async fn returns_error_when_handler_is_missing() {
        let runtime = Runtime::new(
            Arc::new(HandlerRegistry::new()),
            RuntimeContext::new(
                Arc::new(MockClock::new(Timestamp::now_utc())),
                Arc::new(AllowAllAuthorizer),
            ),
            Arc::new(MemoryEventPipeline::new()),
        );

        let error = runtime
            .execute(action("unknown.action", ActorId::new(), SessionId::new()))
            .await
            .expect_err("unregistered action should fail");

        assert!(matches!(error, RuntimeError::HandlerNotFound { .. }));
    }

    #[tokio::test]
    async fn stops_before_handler_when_authorization_is_rejected() {
        let registry = Arc::new(HandlerRegistry::new());
        let handled = Arc::new(AtomicBool::new(false));
        registry
            .register(
                ActionType::new("canvas.draw"),
                Arc::new(handler(handled.clone())),
            )
            .expect("handler registration should succeed");
        let pipeline = Arc::new(MemoryEventPipeline::new());
        let runtime = Runtime::new(
            registry,
            RuntimeContext::new(
                Arc::new(MockClock::new(Timestamp::now_utc())),
                Arc::new(RejectingAuthorizer),
            ),
            pipeline.clone(),
        );

        let error = runtime
            .execute(action("canvas.draw", ActorId::new(), SessionId::new()))
            .await
            .expect_err("rejected action should fail");

        assert!(matches!(error, RuntimeError::AuthorizationFailed { .. }));
        assert!(!handled.load(Ordering::SeqCst));
        assert!(pipeline.events().is_empty());
    }

    #[tokio::test]
    async fn reports_validation_failure_before_authorization() {
        let registry = Arc::new(HandlerRegistry::new());
        registry
            .register(
                ActionType::new("invalid.action"),
                Arc::new(TestHandler {
                    handled: Arc::new(AtomicBool::new(false)),
                    validation_error: Some("invalid action"),
                    handler_error: None,
                }),
            )
            .expect("handler registration should succeed");
        let runtime = Runtime::new(
            registry,
            RuntimeContext::new(
                Arc::new(MockClock::new(Timestamp::now_utc())),
                Arc::new(AllowAllAuthorizer),
            ),
            Arc::new(MemoryEventPipeline::new()),
        );

        let error = runtime
            .execute(action("invalid.action", ActorId::new(), SessionId::new()))
            .await
            .expect_err("invalid action should fail");

        assert!(matches!(error, RuntimeError::ValidationFailed { .. }));
    }
}

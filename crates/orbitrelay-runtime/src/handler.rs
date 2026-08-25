//! Extensible action handlers and their event output drafts.

use async_trait::async_trait;
use orbitrelay_core::Metadata;
use orbitrelay_protocol::{Action, EventType, Payload};

use crate::{HandlerError, RuntimeContext};

/// The handler-produced portion of an event.
///
/// Runtime-owned identity, causality, and timestamp fields are deliberately
/// absent and are assigned only after the handler succeeds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventDraft {
    event_type: EventType,
    payload: Payload,
    metadata: Metadata,
}

impl EventDraft {
    /// Creates an event draft with business-neutral metadata.
    #[must_use]
    pub const fn new(event_type: EventType, payload: Payload, metadata: Metadata) -> Self {
        Self {
            event_type,
            payload,
            metadata,
        }
    }

    /// Returns the event type that the runtime will materialize.
    #[must_use]
    pub const fn event_type(&self) -> &EventType {
        &self.event_type
    }

    /// Returns the draft event payload.
    #[must_use]
    pub const fn payload(&self) -> &Payload {
        &self.payload
    }

    /// Returns the draft event metadata.
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub(crate) fn into_parts(self) -> (EventType, Payload, Metadata) {
        (self.event_type, self.payload, self.metadata)
    }
}

/// Validates and handles one or more registered action types.
#[async_trait]
pub trait ActionHandler: Send + Sync {
    /// Validates action-specific payload and semantics before authorization.
    async fn validate(&self, action: &Action, context: &RuntimeContext)
        -> Result<(), HandlerError>;

    /// Handles an authorized action and returns zero or more event drafts.
    async fn handle(
        &self,
        action: &Action,
        context: &RuntimeContext,
    ) -> Result<Vec<EventDraft>, HandlerError>;
}

#[cfg(test)]
mod tests {
    use orbitrelay_core::Metadata;
    use orbitrelay_protocol::{EventType, Payload};

    use super::EventDraft;

    #[test]
    fn event_draft_contains_only_handler_output() {
        let draft = EventDraft::new(
            EventType::new("document.written"),
            Payload::new(),
            Metadata::new(),
        );

        assert_eq!(draft.event_type().as_str(), "document.written");
        assert_eq!(draft.payload(), &Payload::new());
        assert_eq!(draft.metadata(), &Metadata::new());
    }
}

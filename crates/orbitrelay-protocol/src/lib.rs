//! Transport-independent protocol definitions for OrbitRelay.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod action;
mod actor;
mod event;
mod id;
mod message;
mod payload;
mod session;

pub use action::{Action, ActionType};
pub use actor::{Actor, ActorType};
pub use event::{Event, EventType};
pub use id::{ActionId, ActorId, EventId, MessageId, SessionId};
pub use message::{MessageEnvelope, MessageType};
pub use payload::Payload;
pub use session::Session;

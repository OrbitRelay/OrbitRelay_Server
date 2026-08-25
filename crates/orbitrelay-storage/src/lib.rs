//! Append-only persistence abstractions for OrbitRelay events.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod memory;
mod query;
mod record;
mod store;

pub use error::StorageError;
pub use memory::MemoryEventStore;
pub use query::EventQuery;
pub use record::{EventCursor, EventPage, StoredEvent};
pub use store::EventStore;

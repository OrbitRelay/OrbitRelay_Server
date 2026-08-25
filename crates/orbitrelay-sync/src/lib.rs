//! Transport-independent event propagation and subscription boundaries.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod bus;
mod error;
mod filter;
mod memory;
mod subscription;

pub use bus::EventBus;
pub use error::SyncError;
pub use filter::EventFilter;
pub use memory::{MemoryEventBus, MemorySubscription};
pub use subscription::{Subscription, SubscriptionId};

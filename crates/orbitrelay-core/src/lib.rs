//! Foundational types and utilities shared across OrbitRelay server crates.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod id;
mod metadata;
mod time;
mod version;

pub use error::CoreError;
pub use id::EntityId;
pub use metadata::Metadata;
pub use time::Timestamp;
pub use version::Version;

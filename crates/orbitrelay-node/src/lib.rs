//! Node identity, capability, state, and registration abstractions.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod capability;
mod error;
mod identity;
mod memory;
mod node;
mod registry;
mod state;

pub use capability::Capability;
pub use error::NodeError;
pub use identity::NodeId;
pub use memory::MemoryNodeRegistry;
pub use node::Node;
pub use registry::NodeRegistry;
pub use state::NodeState;

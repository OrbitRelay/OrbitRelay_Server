//! Runtime boundaries for processing OrbitRelay protocol actions.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod context;
mod error;
mod handler;
mod pipeline;
mod registry;
mod runtime;

pub use context::{ActionAuthorizer, Clock, RuntimeContext, SystemClock};
#[cfg(any(test, feature = "test-utils"))]
pub use context::{AllowAllAuthorizer, MockClock};
pub use error::{AuthorizationError, HandlerError, PipelineError, RegistryError, RuntimeError};
pub use handler::{ActionHandler, EventDraft};
pub use pipeline::{EventPipeline, MemoryEventPipeline};
pub use registry::HandlerRegistry;
pub use runtime::Runtime;

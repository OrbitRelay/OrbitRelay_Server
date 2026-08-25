//! Composition root for the OrbitRelay server process.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod bootstrap;
mod config;
mod context;
mod error;
mod health;
mod lifecycle;
mod pipeline;

pub use bootstrap::Bootstrap;
pub use config::ServerConfig;
pub use context::ServerContext;
pub use error::{LifecycleError, ServerError};
pub use health::{HealthState, HealthStatus};
pub use lifecycle::{LifecycleState, ServerLifecycle};
pub use pipeline::PipelineAdapter;

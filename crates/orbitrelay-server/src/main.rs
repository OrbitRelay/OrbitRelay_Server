//! OrbitRelay server process entry point.

use std::sync::Arc;

use async_trait::async_trait;
use orbitrelay_protocol::Action;
use orbitrelay_runtime::{ActionAuthorizer, AuthorizationError};
use orbitrelay_server::{Bootstrap, ServerConfig, ServerError};

struct UnconfiguredAuthorizer;

#[async_trait]
impl ActionAuthorizer for UnconfiguredAuthorizer {
    async fn authorize(&self, _action: &Action) -> Result<(), AuthorizationError> {
        Err(AuthorizationError::new(
            "authorization provider is not configured",
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let config = ServerConfig::load()?;
    let bootstrap = Bootstrap::new(config, Arc::new(UnconfiguredAuthorizer));
    let context = bootstrap.build().await?;

    tokio::signal::ctrl_c()
        .await
        .map_err(|_| ServerError::Shutdown {
            message: "failed to wait for process shutdown signal".to_owned(),
        })?;

    context.shutdown().await
}

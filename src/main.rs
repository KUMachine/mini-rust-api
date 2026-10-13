use mini_rust_api::bootstrap::BootstrapError;
use mini_rust_api::infra::{Config, ConfigError};
use mini_rust_api::presentation::create_router;
use mini_rust_api::presentation::middleware::{CorsLayerError, cors_layer};
use thiserror::Error;

#[derive(Debug, Error)]
enum MainError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Bootstrap(#[from] BootstrapError),

    #[error(transparent)]
    Cors(#[from] CorsLayerError),

    #[error("failed to bind HTTP listener on `{address}`")]
    Bind {
        address: String,
        #[source]
        source: std::io::Error,
    },

    #[error("server failed")]
    Serve(#[source] std::io::Error),
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;

    // Bootstrap: wire up all dependencies
    let state = mini_rust_api::create_app_state(&config).await?;
    let cors = cors_layer(&config)?;

    // Build the HTTP router
    let app = create_router(state).layer(cors);

    // Start the server
    let address = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .map_err(|source| MainError::Bind {
            address: address.clone(),
            source,
        })?;

    tracing::info!(
        "Server is running on: http://{}:{}",
        config.server.host,
        config.server.port
    );

    axum::serve(listener, app).await.map_err(MainError::Serve)?;

    Ok(())
}

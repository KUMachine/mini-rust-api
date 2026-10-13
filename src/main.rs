use mini_rust_api::bootstrap::BootstrapError;
use mini_rust_api::infra::{Config, ConfigError};
use mini_rust_api::presentation::create_router;
use mini_rust_api::presentation::middleware::{CorsLayerError, cors_layer};
use std::error::Error as StdError;
use std::process::ExitCode;
use thiserror::Error;
use tracing_subscriber::EnvFilter;

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
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,tower_http=debug")),
        )
        .init();

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!("{}", error_chain(&error));
            ExitCode::FAILURE
        }
    }
}

/// Render an error with its full chain of causes on one line
fn error_chain(error: &dyn StdError) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

async fn run() -> Result<(), MainError> {
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

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(MainError::Serve)?;

    tracing::info!("Server stopped");

    Ok(())
}

/// Resolve when the process receives Ctrl+C or SIGTERM
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "failed to listen for Ctrl+C");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "failed to listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("Shutdown signal received, draining connections");
}

use axum::{Router, middleware};
use mini_rust_api::bootstrap::BootstrapError;
use mini_rust_api::infra::{Config, ConfigError};
use mini_rust_api::presentation::api::{auth_routes, health_routes, user_routes};
use mini_rust_api::presentation::middleware::{CorsLayerError, auth_middleware, cors_layer};
use mini_rust_api::presentation::openapi::ApiDoc;
use thiserror::Error;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
    let state = mini_rust_api::create_app_state(config.clone()).await?;
    let cors = cors_layer(&config)?;

    // Build the HTTP router
    let app = Router::new()
        .merge(auth_routes())
        .merge(health_routes())
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(user_routes().route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        )))
        .layer(cors)
        .with_state(state);

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

use axum::{Router, middleware};
use mini_rust_api::infra::Config;
use mini_rust_api::presentation::api::{auth_routes, health_routes, user_routes};
use mini_rust_api::presentation::middleware::{auth_middleware, cors_layer};
use mini_rust_api::presentation::openapi::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();

    // Bootstrap: wire up all dependencies
    let state = mini_rust_api::create_app_state(config.clone())
        .await
        .expect("Failed to bootstrap application");

    // Build the HTTP router
    let app = Router::new()
        .merge(auth_routes())
        .merge(health_routes())
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(user_routes().route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        )))
        .layer(cors_layer())
        .with_state(state);

    // Start the server
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
            .await
            .expect("Failed to bind to address");

    tracing::info!(
        "Server is running on: http://{}:{}",
        config.server.host,
        config.server.port
    );

    axum::serve(listener, app)
        .await
        .expect("Server failed unexpectedly");
}

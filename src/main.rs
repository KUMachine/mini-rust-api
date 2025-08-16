use axum::{Router, middleware, routing::get};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use zero2prod::{
    AppState, auth_middleware, auth_routes,
    config::Config,
    docs,
    middleware::{cors::cors_layer, tracing::tracing_layer},
    routes,
};

#[tokio::main]
async fn main() {
    tracing_layer();

    let config = Config::from_env();
    let db = Arc::new(
        zero2prod::config::database::connect()
            .await
            .expect("Failed to connect to database"),
    );

    let state = AppState { db };

    let app = Router::new()
        // Public routes (no authentication required)
        .merge(auth_routes())
        .route("/health", get(routes::health::health_check))
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", docs::ApiDoc::openapi()))
        // Protected routes (authentication required)
        .merge(
            routes::users::routes().route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            )),
        )
        .layer(cors_layer())
        .with_state(state);

    let address = format!("{}:{}", config.server_host, config.server_port);

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("Failed to bind the address");

    tracing::debug!("🚀 Server listening at http://{}", address);
    axum::serve(listener, app).await.unwrap();
}

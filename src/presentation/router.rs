//! HTTP router
//!
//! Assembles all routes, documentation and route-level middleware.

use axum::{Router, middleware};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::api::{auth_routes, health_routes, user_routes};
use super::middleware::auth_middleware;
use super::openapi::ApiDoc;
use super::state::AppState;

/// Build the application router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(auth_routes())
        .merge(health_routes())
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(user_routes().route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        )))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

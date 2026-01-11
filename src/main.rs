use std::sync::Arc;

use axum::{Router, middleware};
use mini_rust_api::app::auth::{LoginUseCase, RegisterUseCase};
use mini_rust_api::app::ports::TokenService;
use mini_rust_api::app::user::{
    CreateUserUseCase, GetUserUseCase, ListUsersUseCase, UpdateUserUseCase,
};
use mini_rust_api::features::user::UserRepository;
use mini_rust_api::infra::auth::JwtTokenService;
use mini_rust_api::infra::config::{self, Config};
use mini_rust_api::infra::persistence::SeaOrmUserRepository;
use mini_rust_api::presentation::AppState;
use mini_rust_api::presentation::api::{auth_routes, health_routes, user_routes};
use mini_rust_api::presentation::middleware::{auth_middleware, cors_layer};
use mini_rust_api::presentation::openapi::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();

    let db = Arc::new(
        config::database::connect()
            .await
            .expect("Failed to connect to database"),
    );

    // Infrastructure layer: Create repository implementation
    let user_repository: Arc<dyn UserRepository> = Arc::new(SeaOrmUserRepository::new(db.clone()));

    // Infrastructure layer: Create token service
    let token_service: Arc<dyn TokenService> = Arc::new(JwtTokenService::new());

    // Application layer: Create use cases
    let login_use_case = Arc::new(LoginUseCase::new(
        user_repository.clone(),
        token_service.clone(),
    ));
    let register_use_case = Arc::new(RegisterUseCase::new(user_repository.clone()));
    let create_user_use_case = Arc::new(CreateUserUseCase::new(user_repository.clone()));
    let get_user_use_case = Arc::new(GetUserUseCase::new(user_repository.clone()));
    let list_users_use_case = Arc::new(ListUsersUseCase::new(user_repository.clone()));
    let update_user_use_case = Arc::new(UpdateUserUseCase::new(user_repository.clone()));

    let state = AppState {
        config: config.clone(),
        db,
        login_use_case,
        register_use_case,
        create_user_use_case,
        get_user_use_case,
        list_users_use_case,
        update_user_use_case,
    };

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

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
            .await
            .unwrap();

    println!(
        "Server is running on: http://{}:{}",
        config.server.host, config.server.port
    );

    axum::serve(listener, app).await.unwrap();
}

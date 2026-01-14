# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust REST API boilerplate using Axum web framework with JWT authentication, PostgreSQL database (via SeaORM), and comprehensive API documentation (via utoipa/Swagger). The codebase follows **Hexagonal Architecture** (Ports & Adapters) with Domain-Driven Design (DDD) principles.

## Common Commands

### Development

```bash
# Run the application
cargo run

# Build the application
cargo build --release

# Format code (uses rustfmt.toml: max_width=100, tab_spaces=4)
cargo fmt

# Run all tests
cargo test

# Run a specific test
cargo test <test_name>

# Run tests without capturing output
cargo test -- --nocapture
```

### Database Migrations

Migrations are managed via SeaORM CLI in the `migration/` workspace member:

```bash
# From the migration directory or use workspace commands
cd migration

# Apply all pending migrations
cargo run

# Generate a new migration
cargo run -- generate MIGRATION_NAME

# Rollback last migration
cargo run -- down

# Check migration status
cargo run -- status

# Fresh install (drop all tables and reapply)
cargo run -- fresh
```

### Docker

```bash
# Start PostgreSQL database
docker-compose up postgresql

# Build and run the API
docker build -t rust-mini-api .
docker-compose up rust-mini-api
```

## Architecture

### Hexagonal Architecture (Ports & Adapters)

The codebase follows hexagonal architecture with clear layer separation:

```
src/
├── domain/              # Core business logic - NO external dependencies
│   ├── shared/          # Shared value objects (UserId)
│   └── user/            # User aggregate
│       ├── entity.rs    # User entity (aggregate root)
│       ├── email.rs     # Email value object
│       ├── password.rs  # Password value object
│       ├── user_profile.rs  # UserProfile value object
│       ├── repository.rs    # Repository trait (port)
│       └── errors.rs    # Domain errors
│
├── app/                 # Use cases, orchestration - depends only on domain
│   ├── auth/            # Auth use cases
│   │   ├── login_use_case.rs
│   │   ├── register_use_case.rs
│   │   └── mod.rs       # Commands & DTOs (LoginCommand, RegisterCommand, AuthToken)
│   ├── user/            # User use cases
│   │   ├── create_user_use_case.rs
│   │   ├── get_user_use_case.rs
│   │   ├── list_users_use_case.rs
│   │   ├── update_user_use_case.rs
│   │   ├── user_response.rs  # Response DTO
│   │   └── mod.rs       # Commands & queries
│   ├── ports/           # Service ports (traits for infrastructure)
│   │   └── token_service.rs  # TokenService trait
│   └── errors.rs        # Application errors (transport-agnostic)
│
├── infra/               # External concerns - adapters
│   ├── persistence/     # Database implementations
│   │   ├── entities/    # SeaORM entities
│   │   └── sea_orm_user_repository.rs  # Repository implementation
│   ├── auth/            # Auth infrastructure
│   │   └── jwt_token_service.rs  # JWT implementation
│   └── config/          # Configuration
│       ├── app_config.rs
│       └── database.rs
│
├── presentation/        # HTTP layer - controllers, routes, extractors
│   ├── api/             # API handlers
│   │   ├── auth.rs      # Auth endpoints
│   │   ├── users.rs     # User CRUD endpoints
│   │   └── health.rs    # Health check
│   ├── errors.rs        # HTTP error responses (IntoResponse)
│   ├── extractors/      # Axum extractors
│   │   ├── validated_json.rs
│   │   └── validated_pagination.rs
│   ├── middleware/      # HTTP middleware
│   │   ├── auth.rs      # JWT authentication
│   │   └── cors.rs      # CORS configuration
│   ├── responses/       # Response types
│   │   ├── api_response.rs  # JSON:API responses
│   │   └── pagination.rs
│   ├── openapi/         # OpenAPI documentation
│   └── state.rs         # AppState (no infrastructure details)
│
├── bootstrap.rs         # Dependency wiring (DI container)
│
└── main.rs              # Minimal application entry point
```

### Layer Dependencies (Hexagonal)

```
                    ┌─────────────────┐
                    │   Presentation  │  ← HTTP adapters (inbound)
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │   Application   │  ← Use cases, orchestration
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │     Domain      │  ← Pure business logic
                    └─────────────────┘
                             ▲
                    ┌────────┴────────┐
                    │  Infrastructure │  ← Database/external adapters (outbound)
                    └─────────────────┘
```

**Key Principles:**

- **Domain Layer**: Pure business logic with no external dependencies. Contains entities, value objects, and repository traits (ports). Has NO knowledge of HTTP, databases, or frameworks.
- **Application Layer**: Orchestrates use cases, depends only on domain. Defines ports (traits) that infrastructure implements. Transport-agnostic - no HTTP concerns.
- **Infrastructure Layer**: Implements adapters - database repositories, external services, configuration. Translates between domain and external systems.
- **Presentation Layer**: Handles HTTP concerns - routes, handlers, extractors, middleware, error-to-HTTP translation. This is where `IntoResponse` implementations live.
- **Bootstrap**: Wires all dependencies together. Isolates DI logic from `main.rs`.

### Key Architectural Decisions

1. **Application errors are transport-agnostic**: `ApplicationError` in `app/errors.rs` knows nothing about HTTP. The `IntoResponse` implementation lives in `presentation/errors.rs`.

2. **AppState doesn't expose infrastructure**: No database connections or concrete implementations in `AppState`. Only use cases are exposed.

3. **Repository errors are persistence-agnostic**: Uses `PersistenceFailure` instead of `DatabaseError` to avoid coupling to specific storage.

4. **Bootstrap module**: All dependency wiring happens in `bootstrap.rs`, keeping `main.rs` clean and focused.

### Dependency Injection

The application uses trait-based dependency injection with Arc-wrapped services:

1. Repository traits (ports) are defined in the domain layer
2. Service traits (ports) are defined in the application layer
3. Implementations (adapters) are in infrastructure
4. Bootstrap wires everything together
5. Use cases are stored in `AppState` and passed via Axum's state system

Example from `bootstrap.rs`:

```rust
// Infrastructure layer: create adapters
let user_repository: Arc<dyn UserRepository> = Arc::new(SeaOrmUserRepository::new(db));
let token_service: Arc<dyn TokenService> = Arc::new(JwtTokenService::new());

// Application layer: inject adapters into use cases
let login_use_case = Arc::new(LoginUseCase::new(user_repository.clone(), token_service.clone()));
```

### Authentication Flow

JWT-based authentication using bcrypt for password hashing:

1. **Registration/Login**: Handled in `presentation/api/auth.rs`
2. **Token Creation**: `infra/auth/jwt_token_service.rs` implements `TokenService` port
3. **Middleware**: `presentation/middleware/auth.rs` validates Bearer tokens
4. **Protected Routes**: Apply `auth_middleware` as a route layer
5. **Access Claims**: Extract `Claims` in handlers using Axum's `FromRequestParts`

### Error Handling

Layered error handling using `thiserror`:

- **DomainError**: Business rule violations (in `domain/user/errors.rs`)
- **RepositoryError**: Persistence errors (in `domain/user/repository.rs`)
- **ApplicationError**: Application layer errors that wrap domain/repo errors (in `app/errors.rs`) - **transport-agnostic**
- **HTTP Translation**: `IntoResponse` for `ApplicationError` lives in `presentation/errors.rs`

All errors return JSON:API formatted responses.

### Validation & Extractors

Custom extractors provide automatic validation:

- **ValidatedJson<T>**: Validates request body using `validator` crate
- **ValidatedPagination<T>**: Validates query parameters
- Both return JSON:API formatted validation errors automatically

### Configuration

Environment-based configuration loaded via dotenvy:

- **Config struct** (`infra/config/app_config.rs`): Loaded at startup
- Required variables: `DATABASE__*`, `JWT_SECRET`
- Optional with defaults: `SERVER__HOST` (0.0.0.0), `SERVER__PORT` (3000)

### API Documentation

OpenAPI/Swagger documentation auto-generated using utoipa:

- **Docs config**: `presentation/openapi/mod.rs` defines `ApiDoc`
- **Endpoint docs**: Use `#[utoipa::path]` macro on handler functions
- **Access**: Swagger UI available at `/api-docs`

## Development Patterns

### Adding a New Feature (Hexagonal Approach)

1. **Domain Layer** (`domain/`):

   - Define entity/aggregate with business logic
   - Create value objects for domain concepts
   - Define repository trait (port)
   - Define domain errors

2. **Application Layer** (`app/`):

   - Create use case(s) that orchestrate domain logic
   - Define command/query DTOs
   - Define response DTOs
   - If needed, define service ports (traits)

3. **Infrastructure Layer** (`infra/`):

   - Implement repository trait with SeaORM
   - Implement any service ports
   - Add SeaORM entities if needed

4. **Presentation Layer** (`presentation/`):

   - Create API handlers
   - Add routes
   - Add error-to-HTTP translation if needed
   - Update OpenAPI documentation

5. **Wire Up** (`bootstrap.rs`):
   - Create infrastructure instances
   - Inject into use cases
   - Add to `AppState`

### Testing

- **Unit tests** for domain logic (in domain modules) - no mocking needed
- **Integration tests** in `tests/` directory
- Use cases can be tested with mock repositories
- Tests can access the library via `use mini_rust_api::*`

## Environment Setup

Required `.env` variables (see `.env.example`):

```
DATABASE__USERNAME=postgres
DATABASE__PASSWORD=your_password
DATABASE__HOST=localhost
DATABASE__PORT=5432
DATABASE__NAME=rust_mini_api
SERVER__HOST=0.0.0.0
SERVER__PORT=3000
JWT_SECRET=your_secret_key
```

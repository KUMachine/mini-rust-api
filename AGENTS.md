# AGENTS.md

Guidance for coding agents working in this repository.

## Project Overview

Rust REST API boilerplate: Axum, JWT authentication, PostgreSQL via SeaORM, OpenAPI docs via utoipa. The code follows **Hexagonal Architecture** (Ports & Adapters) with DDD.

## Commands

```bash
cargo xtask check                 # format, clippy, tests, docs: exactly what CI runs
cargo xtask fix                   # apply rustfmt + machine-applicable clippy fixes
STORAGE=memory cargo run          # run the API without a database (needs JWT_SECRET)
cargo run                         # run the API against PostgreSQL (.env)
cd migration && cargo run -- up   # apply migrations (also: down, status, fresh, generate NAME)
docker compose up                 # PostgreSQL + migrations + API
```

**A change is done when `cargo xtask check` passes.** It needs no database. The toolchain is pinned in `rust-toolchain.toml`, so a local pass means a CI pass. When running cargo directly, pass `--workspace`: the repo root is also a package, so plain `cargo build/test` skips the `migration` and `xtask` crates.

## Architecture

```text
presentation  →  app  →  domain  ←  infra
(HTTP adapter)   (use cases)  (pure logic)  (DB/JWT adapters)
                          bootstrap.rs wires adapters into use cases
```

- **domain/**: entities, value objects, repository traits (ports), domain errors. No framework, HTTP or database knowledge.
- **app/**: one use case per file, command/response DTOs, service ports (`app/ports/`), `ApplicationError`. Transport-agnostic: never import axum here.
- **infra/**: SeaORM and in-memory repositories, `JwtTokenService`, config loading.
- **presentation/**: handlers, `create_router`, extractors, middleware, `IntoResponse` for errors, OpenAPI.
- **bootstrap.rs**: `create_app_state(&Config)` builds the configured storage and seeds the admin; `build_app_state(repo, token_service)` wires use cases from any adapters (tests use it with in-memory ones).

`tests/architecture_test.rs` enforces which modules and crates each layer may reference. When it fails, move the code to the layer its message names; avoid loosening the rule.

### Conventions

1. **`AppState` holds use cases only.** No config, repositories or concrete services. Anything a handler or middleware needs goes through a use case.
2. **Authentication**: `auth_middleware` passes the bearer token to `AuthenticateUseCase`, which calls `TokenService::verify_token` and loads the user's current roles, producing a `CallerContext`. Handlers take `caller: CallerContext` as an extractor. JWTs carry identity only; roles are looked up per request so changes apply immediately.
3. **Authorization lives in use cases**, via `CallerContext::is_admin` / `can_access_user`. It does not live in handlers.
4. **Errors**: domain → `DomainError`, persistence → `RepositoryError` (storage-agnostic, e.g. `Conflict`, `PersistenceFailure`), both wrapped by `ApplicationError`. `presentation/errors.rs` maps them to JSON:API responses. Internal failures are logged with `tracing::error!` and return a generic detail, so database or JWT error text never reaches clients.
5. **Email uniqueness**: use cases pre-check with `exists_with_email`, and also map `RepositoryError::Conflict` from `save` to `EmailAlreadyExists` (409) to cover races. See `app/user/registration.rs::email_conflict`.
6. **bcrypt is CPU-heavy**: wrap hashing and verification in `app::blocking::run_blocking` so it runs off the async workers.
7. **Fallible code returns errors**: `unwrap()` is denied outside tests (workspace clippy lint). Propagate with `?`, or use `expect("why this cannot fail")` for true invariants.
8. **Request bodies**: use `ValidatedJson<T>` (for `T: Validate`) or `JsonBody<T>`, never `axum::Json`, so malformed input returns JSON:API errors.
9. **Repository writes are transactional**: `SeaOrmUserRepository::save` writes the user and its roles in one transaction. Load related data in batches, never per row.

## Adding a Feature

To add a resource, endpoint, use case or entity, follow the `add-feature` skill: `.agents/skills/add-feature/SKILL.md`. It walks the layers inside-out with the User feature as the reference implementation, and defines when the work is done.

## Testing

- **Domain**: unit tests next to the code, no mocks.
- **HTTP**: `tests/api_test.rs` drives the real router with `InMemoryUserRepository` via `tower::ServiceExt::oneshot`. Add a case there for every new endpoint and status code.
- **End to end**: `STORAGE=memory ADMIN_EMAIL=admin@example.com ADMIN_PASSWORD=AdminPass123 cargo run`, then exercise the endpoints with curl (log in as the admin for admin routes). No database or Docker needed.
- **SeaORM queries**: unit tests in the repository module use `sea_orm::MockDatabase` and assert on the transaction log.

## Configuration

Loaded from the environment / `.env` (see `.env.example`) by `Config::from_lookup`, which tests call with a map instead of touching the process environment. Required: `JWT_SECRET` (non-empty; 32+ bytes recommended), plus `DATABASE__USERNAME`, `DATABASE__PASSWORD`, `DATABASE__HOST`, `DATABASE__PORT`, `DATABASE__NAME` when `STORAGE=postgres` (the default). Optional: `STORAGE` (`postgres` | `memory`), `ADMIN_EMAIL` + `ADMIN_PASSWORD` (admin ensured at startup by `EnsureAdminUseCase`; set both or neither), `SERVER__HOST` (0.0.0.0), `SERVER__PORT` (3000), `SERVER__CORS_ALLOWED_ORIGINS` (comma-separated, none by default), `RUST_LOG` (defaults to `info`).

Swagger UI is served at `/api-docs`.

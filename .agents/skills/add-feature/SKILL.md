---
name: add-feature
description: Add a resource, endpoint, use case or entity to this API, layer by layer, with tests. Use when asked to add or extend API functionality (e.g. "add products", "add an endpoint to delete users").
---

# Add a feature

Work **inside-out**: domain, then app, infra, migration, presentation, wiring, tests. Finish each step before starting the next; each ends on a check you can run.

The **reference implementation** is the User feature. Before writing each layer, open its User counterpart (named in each step) and mirror its shape, naming and error handling.

## 1. Domain (`src/domain/<feature>/`)

Reference: `src/domain/user/`.

- Aggregate in `entity.rs`: private fields, a validating constructor (like `User::register`), `reconstitute` for loading from storage, behaviour as methods, getters.
- Value objects that parse on construction (like `Email`, `UserProfile`), so an invalid value cannot exist.
- `DomainError` variants for each rule, with messages written for end users. The HTTP layer uses them as the error detail.
- Repository trait (port) in `repository.rs`: `#[async_trait]`, `Send + Sync`, returning `Result<_, RepositoryError>`.
- Unit tests beside the code for every rule.

Done when `cargo test --lib domain::<feature>` passes.

## 2. Application (`src/app/<feature>/`)

Reference: `src/app/user/`.

- One file per use case, `<verb>_<noun>_use_case.rs`, with `new(Arc<dyn Repo>)` and `execute(command, &CallerContext)`.
- Authorization is the first statement of `execute` (see `get_user_use_case.rs`). Use `CallerContext::is_admin` / `can_access_user`, or add a method there.
- Commands in `mod.rs`: `Deserialize + ToSchema`, plus `Validate` with field rules where the shape alone can be checked.
- Response DTO with `try_from_domain`, deterministic field order and sorting (see `user_response.rs`).
- CPU-heavy work goes through `app::blocking::run_blocking`. Map `RepositoryError::Conflict` from `save` to a specific `ApplicationError` (see `registration.rs::email_conflict`).
- New infrastructure (email, files, queues) gets a port trait in `src/app/ports/`.

Done when it compiles and `cargo test --lib app::<feature>` passes.

## 3. Infrastructure (`src/infra/persistence/`)

Reference: `sea_orm_user_repository.rs`, `in_memory_user_repository.rs`, `entities/users.rs`.

- SeaORM entity in `entities/`, re-exported from `entities/prelude.rs`.
- SeaORM repository: map every `DbErr` through `persistence_error`, write multi-row changes in one transaction, load related rows in batches (one query per relation, never per row).
- In-memory repository enforcing the same unique constraints as the SQL schema, returning `RepositoryError::Conflict`.
- For non-trivial queries, a `MockDatabase` test asserting the statements run (see the tests at the bottom of `sea_orm_user_repository.rs`).

Done when `cargo test --lib infra` passes.

## 4. Migration (`migration/`)

- `cd migration && cargo run -- generate create_<feature>_table`, then add the module to the `Migrator` list in `migration/src/lib.rs`, after the existing ones.
- Mirror every unique index and foreign key the entity declares.

Done when `cargo build -p migration` passes.

## 5. Presentation (`src/presentation/`)

Reference: `api/users.rs`, `router.rs`, `openapi/mod.rs`.

- Handlers in `api/<feature>.rs` with a `<feature>_routes()` function, exported from `api/mod.rs`.
- Merge the routes in `router.rs`. Protected routes go inside an auth `route_layer`, as `user_routes()` does.
- Extract bodies with `ValidatedJson<T>` or `JsonBody<T>`, query strings with `ValidatedPagination<T>`, and the caller with `caller: CallerContext`. Return `201 Created` from creates.
- `#[utoipa::path]` lists **every** status the handler can return, errors with `body = ApiErrorResponse`. Register paths, schemas and a tag in `openapi/mod.rs`.
- New `ApplicationError` variants are mapped in `presentation/errors.rs`.

## 6. Wiring

Reference: `src/bootstrap.rs`, `src/presentation/state.rs`.

- Add each use case to `AppState` and construct it in `build_app_state`.
- A new repository becomes a `build_app_state` parameter, created in `create_app_state` for **both** `Storage::Postgres` and `Storage::Memory`. Update `TestApp::new` in `tests/api_test.rs` to match.

## 7. Tests and verification

- In `tests/api_test.rs`, add a case for every status code listed in each new `#[utoipa::path]`, including the authorization cases (401, 403).
- Run `cargo xtask check`. If `tests/architecture_test.rs` fails, move the code to the layer its message names.
- Smoke test: `STORAGE=memory ADMIN_EMAIL=admin@example.com ADMIN_PASSWORD=AdminPass123 cargo run`, then call each new endpoint with curl.
- If the public API changed, update the API table in `README.md`.

**Done when every documented status code has a matching test case and `cargo xtask check` passes.**

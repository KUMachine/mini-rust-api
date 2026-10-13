# Rust Mini API

A production-ready Rust REST API boilerplate using **Axum**, **SeaORM**, and **JWT authentication**, built with Hexagonal Architecture (Ports & Adapters) and DDD principles.

## Quick Start

```bash
# 1. Setup environment
cp .env.example .env

# 2a. Run everything with Docker (PostgreSQL + migrations + API)
docker compose up --build

# 2b. Or run locally without any database (in-memory, data lost on restart)
STORAGE=memory cargo run

# 2c. Or run locally against the Docker database
docker compose up -d postgresql
(cd migration && cargo run -- up)
cargo run
```

**Swagger UI**: <http://localhost:3000/api-docs>

## Commands

| Command                           | Description                                |
| --------------------------------- | ------------------------------------------ |
| `cargo run`                       | Run the application                        |
| `cargo xtask check`               | Format, lint, test and doc checks (as CI)  |
| `cargo xtask fix`                 | Auto-fix formatting and clippy suggestions |
| `cargo build --release`           | Build for production                       |
| `cd migration && cargo run -- up` | Apply database migrations                  |
| `docker compose up --build`       | Run the full stack in Docker               |

## API

| Method | Path          | Auth          | Description               |
| ------ | ------------- | ------------- | ------------------------- |
| GET    | `/health`     | -             | Health check              |
| POST   | `/register`   | -             | Create an account         |
| POST   | `/login`      | -             | Get a bearer token        |
| GET    | `/users`      | Admin         | List users (paginated)    |
| POST   | `/users`      | Admin         | Create a user             |
| GET    | `/users/{id}` | Owner / Admin | Get a user                |
| PUT    | `/users/{id}` | Owner / Admin | Update a user             |

Errors follow the [JSON:API error format](https://jsonapi.org/format/#errors).

## Project Structure

```text
src/
├── domain/        # Core business logic (entities, value objects, ports)
├── app/           # Use cases & orchestration
├── infra/         # Database, JWT & config adapters
├── presentation/  # HTTP router, handlers, middleware, extractors
└── bootstrap.rs   # Dependency injection wiring
migration/         # SeaORM migrations + CLI
tests/             # HTTP-level integration tests
```

## Documentation

For architecture, conventions and development patterns, see **[AGENTS.md](./AGENTS.md)**.

## License

MIT

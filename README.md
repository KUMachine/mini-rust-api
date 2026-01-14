# Rust Mini API

A production-ready Rust REST API boilerplate using **Axum**, **SeaORM**, and **JWT authentication** — built with Hexagonal Architecture (Ports & Adapters) and DDD principles.

## Quick Start

```bash
# 1. Setup environment
cp .env.example .env

# 2. Start database
docker-compose up postgresql

# 3. Run migrations
cd migration && cargo run && cd ..

# 4. Run the app
cargo run
```

**Swagger UI**: http://localhost:3000/api-docs

## Commands

| Command                           | Description          |
| --------------------------------- | -------------------- |
| `cargo run`                       | Run the application  |
| `cargo build --release`           | Build for production |
| `cargo fmt`                       | Format code          |
| `cargo test`                      | Run all tests        |
| `docker-compose up rust-mini-api` | Run with Docker      |

## Project Structure

```
src/
├── domain/        # Core business logic (entities, value objects, ports)
├── app/           # Use cases & orchestration
├── infra/         # Database & external adapters
├── presentation/  # HTTP handlers, middleware, extractors
└── bootstrap.rs   # Dependency injection wiring
```

## Documentation

For detailed architecture documentation, development patterns, and AI-assisted development guidance, see **[CLAUDE.md](./CLAUDE.md)**.

## License

MIT

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{Database, DbConn};

/// Resolve the database URL.
///
/// `DATABASE_URL` takes precedence; otherwise the URL is assembled from the
/// same `DATABASE__*` variables the API uses.
fn database_url() -> Result<String, DbErr> {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return Ok(url);
    }

    let host = required_env("DATABASE__HOST")?;
    let port = std::env::var("DATABASE__PORT").unwrap_or_else(|_| "5432".to_string());
    let username = required_env("DATABASE__USERNAME")?;
    let password = required_env("DATABASE__PASSWORD")?;
    let name = required_env("DATABASE__NAME")?;

    Ok(format!(
        "postgres://{username}:{password}@{host}:{port}/{name}"
    ))
}

fn required_env(name: &'static str) -> Result<String, DbErr> {
    std::env::var(name).map_err(|_| DbErr::Custom(format!("{name} is required")))
}

async fn connect() -> Result<DbConn, DbErr> {
    Database::connect(database_url()?).await
}

#[tokio::main]
async fn main() {
    cli::run_cli_with_custom_connection(migration::Migrator, connect).await;
}

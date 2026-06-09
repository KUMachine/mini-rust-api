use sea_orm_migration::prelude::*;

#[derive(Debug)]
enum MigrationEnvError {
    MissingEnv {
        name: &'static str,
        source: std::env::VarError,
    },
}

impl std::fmt::Display for MigrationEnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEnv { name, .. } => write!(f, "{name} is required"),
        }
    }
}

impl std::error::Error for MigrationEnvError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingEnv { source, .. } => Some(source),
        }
    }
}

#[cfg(debug_assertions)]
fn load_env() -> Result<(), MigrationEnvError> {
    dotenvy::dotenv().ok();

    let database_host = required_env("DATABASE__HOST")?;
    let database_port = std::env::var("DATABASE__PORT").unwrap_or_else(|_| "5432".to_string());
    let database_user = required_env("DATABASE__USERNAME")?;
    let database_password = required_env("DATABASE__PASSWORD")?;
    let database_name = required_env("DATABASE__NAME")?;

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        database_user, database_password, database_host, database_port, database_name
    );

    std::env::set_var("DATABASE_URL", database_url);
    Ok(())
}

#[cfg(not(debug_assertions))]
fn load_env() -> Result<(), MigrationEnvError> {
    Ok(())
}

#[cfg(debug_assertions)]
fn required_env(name: &'static str) -> Result<String, MigrationEnvError> {
    std::env::var(name).map_err(|source| MigrationEnvError::MissingEnv { name, source })
}

#[async_std::main]
async fn main() {
    if let Err(error) = load_env() {
        eprintln!("{error}");
        std::process::exit(1);
    }

    cli::run_cli(migration::Migrator).await;
}

use sea_orm::{Database, DbConn};

use crate::config::Config;

pub async fn connect() -> Result<DbConn, sea_orm::DbErr> {
    let config = Config::from_env();
    Database::connect(&config.database_url).await
}

use sea_orm::{Database, DbConn, DbErr};

use crate::config::Config;

pub async fn connect() -> Result<DbConn, DbErr> {
    let config = Config::from_env();
    Database::connect(&config.database_url).await
}

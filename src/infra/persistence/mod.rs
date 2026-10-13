pub mod entities;
pub mod in_memory_user_repository;
pub mod sea_orm_user_repository;

pub use in_memory_user_repository::InMemoryUserRepository;
pub use sea_orm_user_repository::SeaOrmUserRepository;

use crate::domain::user::repository::RepositoryError;
use sea_orm::{DbErr, SqlErr};

/// Translate a SeaORM error into a persistence-agnostic repository error
///
/// Shared by every SeaORM repository so unique violations always surface as `Conflict`.
pub(crate) fn persistence_error(err: DbErr) -> RepositoryError {
    match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(detail)) => RepositoryError::Conflict(detail),
        _ => RepositoryError::PersistenceFailure(err.to_string()),
    }
}

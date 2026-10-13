//! User registration shared by self-service sign-up and admin user creation

use super::UserResponse;
use crate::app::blocking::run_blocking;
use crate::app::errors::{AppResult, ApplicationError};
use crate::domain::user::repository::RepositoryError;
use crate::domain::user::{Email, User, UserRepository};

/// Data required to register a user
pub(crate) struct NewUser {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub age: u8,
}

/// Validate, hash and persist a new user
pub(crate) async fn register_user(
    user_repository: &dyn UserRepository,
    new_user: NewUser,
) -> AppResult<UserResponse> {
    // Parse and validate email (domain validation)
    let email = Email::try_from(new_user.email.as_str())?;

    // Check if user already exists
    if user_repository.exists_with_email(&email).await? {
        return Err(ApplicationError::EmailAlreadyExists(email.to_string()));
    }

    // Create user entity (domain logic); bcrypt hashing is CPU-heavy
    let NewUser {
        password,
        first_name,
        last_name,
        age,
        ..
    } = new_user;
    let register_email = email.clone();
    let mut user =
        run_blocking(move || User::register(register_email, password, first_name, last_name, age))
            .await??;

    // Persist the user; a concurrent registration may still win the race
    user_repository
        .save(&mut user)
        .await
        .map_err(|e| email_conflict(e, &email))?;

    UserResponse::try_from_domain(&user)
}

/// Map a uniqueness conflict on save to `EmailAlreadyExists`
pub(crate) fn email_conflict(error: RepositoryError, email: &Email) -> ApplicationError {
    match error {
        RepositoryError::Conflict(_) => ApplicationError::EmailAlreadyExists(email.to_string()),
        other => other.into(),
    }
}

use super::{AuthToken, LoginCommand};
use crate::app::blocking::run_blocking;
use crate::app::errors::{AppResult, ApplicationError};
use crate::app::ports::TokenService;
use crate::domain::user::{Email, Password, UserRepository};
use std::sync::{Arc, LazyLock};

/// Hash verified when the email is unknown, so a failed login takes the same
/// time whether or not the account exists (prevents user enumeration)
static DUMMY_PASSWORD: LazyLock<Password> = LazyLock::new(|| {
    Password::hash("DummyPassword1").expect("hashing a constant valid password cannot fail")
});

/// LoginUseCase - handles user login
pub struct LoginUseCase {
    user_repository: Arc<dyn UserRepository>,
    token_service: Arc<dyn TokenService>,
}

impl LoginUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        token_service: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            user_repository,
            token_service,
        }
    }

    pub async fn execute(&self, command: LoginCommand) -> AppResult<AuthToken> {
        // Parse and validate email (domain validation)
        let email = Email::try_from(command.email)?;
        let password = command.password;

        // Find user by email
        let Some(user) = self.user_repository.find_by_email(&email).await? else {
            run_blocking(move || DUMMY_PASSWORD.verify(&password)).await?;
            return Err(ApplicationError::InvalidCredentials);
        };

        // Domain logic: authenticate user (bcrypt verification is CPU-heavy)
        let user = run_blocking(move || user.authenticate(&password).map(|()| user))
            .await?
            .map_err(|_| ApplicationError::InvalidCredentials)?;

        // Infrastructure concern: generate token (identity only, no roles)
        let user_id = user.id().ok_or(ApplicationError::MissingUserId)?;
        let token = self
            .token_service
            .generate_token(user_id.value(), user.email().as_ref())
            .await?;

        Ok(AuthToken::new(token))
    }
}

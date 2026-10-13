use super::registration::email_conflict;
use crate::app::blocking::run_blocking;
use crate::app::errors::AppResult;
use crate::domain::user::{Email, Role, User, UserRepository};
use std::sync::Arc;

/// Profile given to a seeded admin; the domain requires a name and an adult age
const SEED_FIRST_NAME: &str = "Admin";
const SEED_LAST_NAME: &str = "User";
const SEED_AGE: u8 = 18;

/// What `EnsureAdminUseCase` did
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnsureAdminOutcome {
    /// No user had the email; an admin was created with the given password
    Created,
    /// An existing user was given the admin role; their password is unchanged
    Promoted,
    /// The user was already an admin; nothing changed
    AlreadyAdmin,
}

/// EnsureAdminUseCase - guarantees an admin account exists (startup seeding)
///
/// Idempotent, so it is safe to run on every start.
pub struct EnsureAdminUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl EnsureAdminUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, email: &str, password: &str) -> AppResult<EnsureAdminOutcome> {
        let email = Email::try_from(email)?;

        let (mut user, outcome) = match self.user_repository.find_by_email(&email).await? {
            Some(user) if user.has_role(&Role::Admin) => {
                return Ok(EnsureAdminOutcome::AlreadyAdmin);
            }
            Some(user) => (user, EnsureAdminOutcome::Promoted),
            None => {
                let register_email = email.clone();
                let password = password.to_string();
                let user = run_blocking(move || {
                    User::register(
                        register_email,
                        password,
                        SEED_FIRST_NAME.to_string(),
                        SEED_LAST_NAME.to_string(),
                        SEED_AGE,
                    )
                })
                .await??;
                (user, EnsureAdminOutcome::Created)
            }
        };

        user.add_role(Role::Admin);
        self.user_repository
            .save(&mut user)
            .await
            .map_err(|e| email_conflict(e, &email))?;

        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::persistence::InMemoryUserRepository;

    const PASSWORD: &str = "SecurePass123";

    async fn find(repository: &InMemoryUserRepository, email: &str) -> User {
        repository
            .find_by_email(&Email::try_from(email).unwrap())
            .await
            .unwrap()
            .unwrap()
    }

    #[tokio::test]
    async fn test_creates_admin_then_is_idempotent() {
        let repository = Arc::new(InMemoryUserRepository::new());
        let use_case = EnsureAdminUseCase::new(repository.clone());

        let outcome = use_case.execute("admin@example.com", PASSWORD).await;
        assert_eq!(outcome.unwrap(), EnsureAdminOutcome::Created);

        let admin = find(&repository, "admin@example.com").await;
        assert!(admin.has_role(&Role::Admin));
        assert!(admin.authenticate(PASSWORD).is_ok());

        let outcome = use_case.execute("admin@example.com", PASSWORD).await;
        assert_eq!(outcome.unwrap(), EnsureAdminOutcome::AlreadyAdmin);
    }

    #[tokio::test]
    async fn test_promotes_existing_user_without_changing_password() {
        let repository = Arc::new(InMemoryUserRepository::new());
        let mut user = User::register(
            Email::try_from("existing@example.com").unwrap(),
            PASSWORD.to_string(),
            "Existing".to_string(),
            "User".to_string(),
            30,
        )
        .unwrap();
        repository.save(&mut user).await.unwrap();

        let outcome = EnsureAdminUseCase::new(repository.clone())
            .execute("existing@example.com", "DifferentPass456")
            .await;
        assert_eq!(outcome.unwrap(), EnsureAdminOutcome::Promoted);

        let promoted = find(&repository, "existing@example.com").await;
        assert!(promoted.has_role(&Role::Admin));
        assert!(promoted.has_role(&Role::User));
        assert!(promoted.authenticate(PASSWORD).is_ok());
        assert_eq!(promoted.profile().first_name(), "Existing");
    }
}

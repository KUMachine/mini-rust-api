use super::{DomainError, Email, Password, UserProfile};
use crate::features::shared::UserId;
use chrono::NaiveDate;

/// User aggregate root - rich domain entity with business logic
#[derive(Clone)]
pub struct User {
    id: Option<UserId>,
    email: Email,
    password: Password,
    profile: UserProfile,
    created_at: NaiveDate,
}

impl User {
    /// Register a new user (factory method)
    pub fn register(
        email: Email,
        raw_password: String,
        first_name: String,
        last_name: String,
        age: u8,
    ) -> Result<Self, DomainError> {
        let password = Password::hash(raw_password)?;
        let profile = UserProfile::new(first_name, last_name, age)?;

        Ok(Self {
            id: None,
            email,
            password,
            profile,
            created_at: chrono::Utc::now().naive_utc().date(),
        })
    }

    /// Reconstitute a User from persistence (not a business operation)
    /// This is used when loading from the database
    pub fn reconstitute(
        id: UserId,
        email: Email,
        password: Password,
        profile: UserProfile,
        created_at: NaiveDate,
    ) -> Self {
        Self {
            id: Some(id),
            email,
            password,
            profile,
            created_at,
        }
    }

    /// Authenticate the user with a raw password
    /// This is a domain behavior
    pub fn authenticate(&self, raw_password: &str) -> Result<(), DomainError> {
        if !self.password.verify(raw_password) {
            return Err(DomainError::InvalidCredentials);
        }
        Ok(())
    }

    /// Change the user's email
    /// This is a domain behavior with business rules
    pub fn change_email(&mut self, new_email: Email) -> Result<(), DomainError> {
        // Business rule: email must be different
        if self.email == new_email {
            // Email is the same, no change needed
            return Ok(());
        }

        self.email = new_email;
        Ok(())
    }

    /// Update the user's profile
    pub fn update_profile(
        &mut self,
        first_name: String,
        last_name: String,
        age: u8,
    ) -> Result<(), DomainError> {
        self.profile.update(first_name, last_name, age)?;
        Ok(())
    }

    /// Change password
    pub fn change_password(&mut self, raw_password: String) -> Result<(), DomainError> {
        self.password = Password::hash(raw_password)?;
        Ok(())
    }

    // Getters - domain entities control access to their data

    pub fn id(&self) -> Option<UserId> {
        self.id
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password(&self) -> &Password {
        &self.password
    }

    pub fn profile(&self) -> &UserProfile {
        &self.profile
    }

    pub fn created_at(&self) -> NaiveDate {
        self.created_at
    }

    /// Set the ID (used after persistence)
    pub(crate) fn set_id(&mut self, id: UserId) {
        self.id = Some(id);
    }
}

// Note: We implement Debug carefully to avoid logging sensitive data
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .field("profile", &self.profile)
            .field("created_at", &self.created_at)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_registration() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let user = User::register(
            email,
            "SecurePass123".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            25,
        )
        .unwrap();

        assert!(user.id().is_none());
        assert_eq!(user.email().as_str(), "test@example.com");
        assert_eq!(user.profile().first_name(), "John");
    }

    #[test]
    fn test_user_authentication() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let user = User::register(
            email,
            "SecurePass123".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            25,
        )
        .unwrap();

        assert!(user.authenticate("SecurePass123").is_ok());
        assert!(user.authenticate("WrongPassword").is_err());
    }

    #[test]
    fn test_user_update_profile() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let mut user = User::register(
            email,
            "SecurePass123".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            25,
        )
        .unwrap();

        user.update_profile("Jane".to_string(), "Smith".to_string(), 30)
            .unwrap();

        assert_eq!(user.profile().first_name(), "Jane");
        assert_eq!(user.profile().last_name(), "Smith");
        assert_eq!(user.profile().age(), 30);
    }

    #[test]
    fn test_user_change_email() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let mut user = User::register(
            email,
            "SecurePass123".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            25,
        )
        .unwrap();

        let new_email = Email::new("newemail@example.com".to_string()).unwrap();
        user.change_email(new_email).unwrap();

        assert_eq!(user.email().as_str(), "newemail@example.com");
    }
}

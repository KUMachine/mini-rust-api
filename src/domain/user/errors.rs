use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DomainError {
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Password must be at least 8 characters long")]
    PasswordTooShort,

    #[error(
        "Password must contain at least one uppercase letter, one lowercase letter, and one number"
    )]
    PasswordTooWeak,

    #[error("Failed to hash password")]
    PasswordHashingFailed,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User must be at least 18 years old")]
    UserTooYoung,

    #[error("Invalid age: must be between 18 and 150")]
    InvalidAge,

    #[error("First name cannot be empty")]
    EmptyFirstName,

    #[error("Last name cannot be empty")]
    EmptyLastName,

    #[error("User with email {0} already exists")]
    EmailAlreadyExists(String),
}

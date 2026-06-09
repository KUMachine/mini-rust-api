use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DomainError {
    #[error("invalid email format: {0}")]
    InvalidEmail(String),

    #[error("password must be at least 8 characters long")]
    PasswordTooShort,

    #[error(
        "password must contain at least one uppercase letter, one lowercase letter, and one number"
    )]
    PasswordTooWeak,

    #[error("failed to hash password")]
    PasswordHashingFailed,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("user must be at least 18 years old")]
    UserTooYoung,

    #[error("invalid age: must be between 18 and 150")]
    InvalidAge,

    #[error("first name cannot be empty")]
    EmptyFirstName,

    #[error("last name cannot be empty")]
    EmptyLastName,

    #[error("user with email {0} already exists")]
    EmailAlreadyExists(String),
}

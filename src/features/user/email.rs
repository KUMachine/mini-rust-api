use std::fmt::Display;

use super::errors::DomainError;
use serde::{Deserialize, Serialize};
use validator::ValidateEmail;

/// Email value object - ensures email validity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl TryFrom<String> for Email {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim().to_lowercase();

        if !ValidateEmail::validate_email(&trimmed) {
            return Err(DomainError::InvalidEmail(value));
        }

        Ok(Self(trimmed))
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(Email::try_from("test@example.com".to_string()).is_ok());
        assert!(Email::try_from("user.name@domain.co.uk".to_string()).is_ok());
    }

    #[test]
    fn test_invalid_email() {
        assert!(Email::try_from("".to_string()).is_err());
        assert!(Email::try_from("invalid".to_string()).is_err());
        assert!(Email::try_from("@example.com".to_string()).is_err());
        assert!(Email::try_from("user@".to_string()).is_err());
        assert!(Email::try_from("user@domain".to_string()).is_err());
    }

    #[test]
    fn test_email_normalization() {
        let email = Email::try_from("  TEST@EXAMPLE.COM  ".to_string()).unwrap();
        assert_eq!(email.0, "test@example.com");
    }
}

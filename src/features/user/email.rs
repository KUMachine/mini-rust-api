use super::errors::DomainError;
use serde::{Deserialize, Serialize};

/// Email value object - ensures email validity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    /// Create a new Email, validating the format
    pub fn new(value: String) -> Result<Self, DomainError> {
        let trimmed = value.trim().to_lowercase();

        if !Self::is_valid(&trimmed) {
            return Err(DomainError::InvalidEmail(value));
        }

        Ok(Self(trimmed))
    }

    /// Validate email format
    fn is_valid(email: &str) -> bool {
        // Basic email validation
        if email.is_empty() || email.len() < 3 {
            return false;
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let local = parts[0];
        let domain = parts[1];

        if local.is_empty() || domain.is_empty() {
            return false;
        }

        // Domain must have at least one dot
        if !domain.contains('.') {
            return false;
        }

        // Domain parts must not be empty
        let domain_parts: Vec<&str> = domain.split('.').collect();
        if domain_parts.iter().any(|part| part.is_empty()) {
            return false;
        }

        true
    }

    /// Get the email as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.0
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
        assert!(Email::new("test@example.com".to_string()).is_ok());
        assert!(Email::new("user.name@domain.co.uk".to_string()).is_ok());
    }

    #[test]
    fn test_invalid_email() {
        assert!(Email::new("".to_string()).is_err());
        assert!(Email::new("invalid".to_string()).is_err());
        assert!(Email::new("@example.com".to_string()).is_err());
        assert!(Email::new("user@".to_string()).is_err());
        assert!(Email::new("user@domain".to_string()).is_err());
    }

    #[test]
    fn test_email_normalization() {
        let email = Email::new("  TEST@EXAMPLE.COM  ".to_string()).unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }
}

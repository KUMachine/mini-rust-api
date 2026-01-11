use super::errors::DomainError;
use bcrypt::{DEFAULT_COST, hash, verify};

/// Password value object - handles hashing and verification
#[derive(Clone)]
pub struct Password {
    hashed: String,
}

impl Password {
    /// Create a new Password by hashing a raw password
    pub fn hash(raw: String) -> Result<Self, DomainError> {
        // Validate password strength
        Self::validate_strength(&raw)?;

        let hashed = hash(raw, DEFAULT_COST).map_err(|_| DomainError::PasswordHashingFailed)?;

        Ok(Self { hashed })
    }

    /// Create a Password from an already hashed value (e.g., from database)
    pub fn from_hash(hashed: String) -> Self {
        Self { hashed }
    }

    /// Verify a raw password against this hashed password
    pub fn verify(&self, raw_password: &str) -> bool {
        verify(raw_password, &self.hashed).unwrap_or(false)
    }

    /// Get the hashed password string
    pub fn hashed(&self) -> &str {
        &self.hashed
    }

    /// Validate password strength rules
    fn validate_strength(password: &str) -> Result<(), DomainError> {
        if password.len() < 8 {
            return Err(DomainError::PasswordTooShort);
        }

        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());

        if !has_uppercase || !has_lowercase || !has_digit {
            return Err(DomainError::PasswordTooWeak);
        }

        Ok(())
    }
}

// Note: We don't implement Debug or Display to avoid accidentally logging passwords
impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Password([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let raw = "SecurePass123".to_string();
        let password = Password::hash(raw.clone()).unwrap();
        assert!(password.verify(&raw));
        assert!(!password.verify("WrongPassword"));
    }

    #[test]
    fn test_password_too_short() {
        let result = Password::hash("Short1".to_string());
        assert!(matches!(result, Err(DomainError::PasswordTooShort)));
    }

    #[test]
    fn test_password_too_weak() {
        let result = Password::hash("alllowercase".to_string());
        assert!(matches!(result, Err(DomainError::PasswordTooWeak)));
    }

    #[test]
    fn test_password_from_hash() {
        let raw = "ValidPass123".to_string();
        let password = Password::hash(raw.clone()).unwrap();
        let hash_string = password.hashed().to_string();

        let password_from_hash = Password::from_hash(hash_string);
        assert!(password_from_hash.verify(&raw));
    }
}

use super::errors::DomainError;

/// UserProfile value object - encapsulates user profile information
#[derive(Debug, Clone)]
pub struct UserProfile {
    first_name: String,
    last_name: String,
    age: u8,
}

impl UserProfile {
    fn validate_and_normalize(
        first_name: String,
        last_name: String,
        age: u8,
    ) -> Result<(String, String, u8), DomainError> {
        let first_name = first_name.trim().to_string();
        let last_name = last_name.trim().to_string();

        if first_name.is_empty() {
            return Err(DomainError::EmptyFirstName);
        }

        if last_name.is_empty() {
            return Err(DomainError::EmptyLastName);
        }

        if age < 18 {
            return Err(DomainError::UserTooYoung);
        }

        if age > 150 {
            return Err(DomainError::InvalidAge);
        }

        Ok((first_name, last_name, age))
    }

    /// Create a new UserProfile with validation
    pub fn new(first_name: String, last_name: String, age: u8) -> Result<Self, DomainError> {
        let (first_name, last_name, age) =
            Self::validate_and_normalize(first_name, last_name, age)?;

        Ok(Self {
            first_name,
            last_name,
            age,
        })
    }

    /// Get the first name
    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    /// Get the last name
    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    /// Get the age
    pub fn age(&self) -> u8 {
        self.age
    }

    /// Get the full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Update the profile
    pub fn update(
        &mut self,
        first_name: String,
        last_name: String,
        age: u8,
    ) -> Result<(), DomainError> {
        let (first_name, last_name, age) =
            Self::validate_and_normalize(first_name, last_name, age)?;

        self.first_name = first_name;
        self.last_name = last_name;
        self.age = age;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_profile() {
        let profile = UserProfile::new("John".to_string(), "Doe".to_string(), 25).unwrap();
        assert_eq!(profile.first_name(), "John");
        assert_eq!(profile.last_name(), "Doe");
        assert_eq!(profile.age(), 25);
        assert_eq!(profile.full_name(), "John Doe");
    }

    #[test]
    fn test_empty_first_name() {
        let result = UserProfile::new("".to_string(), "Doe".to_string(), 25);
        assert!(matches!(result, Err(DomainError::EmptyFirstName)));
    }

    #[test]
    fn test_user_too_young() {
        let result = UserProfile::new("John".to_string(), "Doe".to_string(), 17);
        assert!(matches!(result, Err(DomainError::UserTooYoung)));
    }

    #[test]
    fn test_profile_update() {
        let mut profile = UserProfile::new("John".to_string(), "Doe".to_string(), 25).unwrap();
        profile
            .update("Jane".to_string(), "Smith".to_string(), 30)
            .unwrap();
        assert_eq!(profile.first_name(), "Jane");
        assert_eq!(profile.full_name(), "Jane Smith");
    }
}

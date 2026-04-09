use std::fmt;
use std::str::FromStr;

/// User role - a fixed set of roles known at compile time
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    User,
}

impl Role {
    /// Returns the string representation used for persistence and JWT claims
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::User => "user",
        }
    }
}

impl Default for Role {
    fn default() -> Self {
        Self::User
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "user" => Ok(Role::User),
            other => Err(format!("Unknown role: {}", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_display() {
        assert_eq!(Role::Admin.to_string(), "admin");
        assert_eq!(Role::User.to_string(), "user");
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("admin").unwrap(), Role::Admin);
        assert_eq!(Role::from_str("Admin").unwrap(), Role::Admin);
        assert_eq!(Role::from_str("user").unwrap(), Role::User);
        assert!(Role::from_str("unknown").is_err());
    }

    #[test]
    fn test_role_default() {
        assert_eq!(Role::default(), Role::User);
    }

    #[test]
    fn test_role_hashset() {
        use std::collections::HashSet;
        let mut roles = HashSet::new();
        roles.insert(Role::Admin);
        roles.insert(Role::User);
        roles.insert(Role::Admin); // duplicate
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&Role::Admin));
    }
}

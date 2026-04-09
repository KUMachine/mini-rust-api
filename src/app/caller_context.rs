//! Caller context for authorization
//!
//! Represents the identity and roles of the authenticated caller.
//! Built by the auth middleware from the JWT token + a DB role lookup,
//! then inserted into request extensions for handler extraction.

use crate::domain::user::Role;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum_core::extract::FromRequestParts;
use std::collections::HashSet;

/// Context about the authenticated caller, passed to use cases for authorization
#[derive(Debug, Clone)]
pub struct CallerContext {
    pub user_id: i32,
    pub roles: HashSet<Role>,
}

impl CallerContext {
    pub fn new(user_id: i32, roles: HashSet<Role>) -> Self {
        Self { user_id, roles }
    }

    /// Check if the caller has a specific role
    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    /// Check if the caller is an admin
    pub fn is_admin(&self) -> bool {
        self.roles.contains(&Role::Admin)
    }

    /// Check if the caller owns the resource (i.e., the resource belongs to them)
    pub fn is_owner(&self, resource_user_id: i32) -> bool {
        self.user_id == resource_user_id
    }

    /// Check if the caller can access a user resource (admin OR owner)
    pub fn can_access_user(&self, target_user_id: i32) -> bool {
        self.is_admin() || self.is_owner(target_user_id)
    }
}

/// Extract CallerContext from request extensions (inserted by auth middleware)
impl<S> FromRequestParts<S> for CallerContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CallerContext>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_can_access_any_user() {
        let caller = CallerContext::new(1, HashSet::from([Role::Admin, Role::User]));
        assert!(caller.can_access_user(1));
        assert!(caller.can_access_user(999));
        assert!(caller.is_admin());
    }

    #[test]
    fn test_user_can_only_access_own_data() {
        let caller = CallerContext::new(1, HashSet::from([Role::User]));
        assert!(caller.can_access_user(1));
        assert!(!caller.can_access_user(2));
        assert!(!caller.is_admin());
    }

    #[test]
    fn test_is_owner() {
        let caller = CallerContext::new(42, HashSet::from([Role::User]));
        assert!(caller.is_owner(42));
        assert!(!caller.is_owner(43));
    }
}

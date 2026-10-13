//! In-memory implementation of UserRepository
//!
//! Useful for tests and for running the API without a database.
//! Enforces the same email uniqueness rule as the SQL schema.

use crate::domain::shared::UserId;
use crate::domain::user::repository::{RepositoryError, UserRepository};
use crate::domain::user::{Email, Role, User};
use async_trait::async_trait;
use std::collections::{BTreeMap, HashSet};
use std::sync::{Mutex, MutexGuard};

#[derive(Default)]
struct Store {
    last_id: i32,
    users: BTreeMap<i32, User>,
}

/// UserRepository backed by a process-local map
#[derive(Default)]
pub struct InMemoryUserRepository {
    store: Mutex<Store>,
}

impl InMemoryUserRepository {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn store(&self) -> Result<MutexGuard<'_, Store>, RepositoryError> {
        self.store
            .lock()
            .map_err(|_| RepositoryError::Unexpected("in-memory store lock poisoned".to_string()))
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        Ok(self.store()?.users.get(&id.value()).cloned())
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .store()?
            .users
            .values()
            .find(|user| user.email() == email)
            .cloned())
    }

    async fn save(&self, user: &mut User) -> Result<(), RepositoryError> {
        let mut store = self.store()?;

        let id = match user.id() {
            Some(id) if !store.users.contains_key(&id.value()) => {
                return Err(RepositoryError::NotFound);
            }
            Some(id) => id.value(),
            None => store.last_id + 1,
        };

        let email_taken = store
            .users
            .iter()
            .any(|(other_id, other)| *other_id != id && other.email() == user.email());
        if email_taken {
            return Err(RepositoryError::Conflict(format!(
                "email {} already exists",
                user.email()
            )));
        }

        store.last_id = store.last_id.max(id);
        user.set_id(UserId::from(id));
        store.users.insert(id, user.clone());

        Ok(())
    }

    async fn exists_with_email(&self, email: &Email) -> Result<bool, RepositoryError> {
        Ok(self.find_by_email(email).await?.is_some())
    }

    async fn list(
        &self,
        page: u64,
        rows_per_page: u64,
    ) -> Result<(Vec<User>, u64), RepositoryError> {
        let store = self.store()?;
        let offset = page.saturating_sub(1).saturating_mul(rows_per_page);

        let users = store
            .users
            .values()
            .rev()
            .skip(usize::try_from(offset).unwrap_or(usize::MAX))
            .take(usize::try_from(rows_per_page).unwrap_or(usize::MAX))
            .cloned()
            .collect();

        Ok((users, store.users.len() as u64))
    }

    async fn find_roles_by_user_id(&self, id: UserId) -> Result<HashSet<Role>, RepositoryError> {
        Ok(self
            .store()?
            .users
            .get(&id.value())
            .map(|user| user.roles().clone())
            .unwrap_or_default())
    }
}

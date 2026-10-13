use super::entities::roles::{self, Entity as RolesEntity};
use super::entities::user_roles::{self, Entity as UserRolesEntity};
use super::entities::users::{self, Entity as UsersEntity};
use crate::domain::shared::UserId;
use crate::domain::user::entity::User;
use crate::domain::user::repository::{RepositoryError, UserRepository};
use crate::domain::user::{Email, Password, Role, UserProfile};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, SqlErr, TransactionTrait,
};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

/// SeaORM implementation of UserRepository
pub struct SeaOrmUserRepository {
    db: Arc<sea_orm::DatabaseConnection>,
}

/// Translate a SeaORM error into a persistence-agnostic repository error
fn persistence_error(err: DbErr) -> RepositoryError {
    match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(detail)) => RepositoryError::Conflict(detail),
        _ => RepositoryError::PersistenceFailure(err.to_string()),
    }
}

impl SeaOrmUserRepository {
    pub fn new(db: Arc<sea_orm::DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Load roles for the given user IDs from the junction table in a single query
    async fn load_roles(
        &self,
        user_ids: &[i32],
    ) -> Result<HashMap<i32, HashSet<Role>>, RepositoryError> {
        if user_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = UserRolesEntity::find()
            .filter(user_roles::Column::UserId.is_in(user_ids.iter().copied()))
            .find_also_related(RolesEntity)
            .all(self.db.as_ref())
            .await
            .map_err(persistence_error)?;

        let mut roles: HashMap<i32, HashSet<Role>> = HashMap::new();
        for (user_role, role_model) in rows {
            if let Some(role_model) = role_model
                && let Ok(role) = Role::from_str(&role_model.name)
            {
                roles.entry(user_role.user_id).or_default().insert(role);
            }
        }

        Ok(roles)
    }

    /// Replace a user's role assignments
    async fn save_roles<C: ConnectionTrait>(
        conn: &C,
        user_id: i32,
        roles: &HashSet<Role>,
    ) -> Result<(), RepositoryError> {
        UserRolesEntity::delete_many()
            .filter(user_roles::Column::UserId.eq(user_id))
            .exec(conn)
            .await
            .map_err(persistence_error)?;

        if roles.is_empty() {
            return Ok(());
        }

        let role_models = RolesEntity::find()
            .filter(roles::Column::Name.is_in(roles.iter().map(Role::as_str)))
            .all(conn)
            .await
            .map_err(persistence_error)?;

        if role_models.len() != roles.len() {
            return Err(RepositoryError::PersistenceFailure(format!(
                "some of the roles {roles:?} are missing from the roles table"
            )));
        }

        UserRolesEntity::insert_many(role_models.into_iter().map(|role| user_roles::ActiveModel {
            user_id: Set(user_id),
            role_id: Set(role.id),
        }))
        .exec_without_returning(conn)
        .await
        .map_err(persistence_error)?;

        Ok(())
    }

    /// Convert SeaORM model to domain User entity
    fn to_domain(model: users::Model, roles: HashSet<Role>) -> Result<User, RepositoryError> {
        let email = Email::try_from(model.email)
            .map_err(|e| RepositoryError::PersistenceFailure(format!("invalid email: {e}")))?;

        let password = Password::from_hash(model.password_hash);

        let age = u8::try_from(model.age).map_err(|_| {
            RepositoryError::PersistenceFailure(format!("invalid age: {}", model.age))
        })?;

        let profile = UserProfile::new(model.first_name, model.last_name, age)
            .map_err(|e| RepositoryError::PersistenceFailure(format!("invalid profile: {e}")))?;

        Ok(User::reconstitute(
            UserId::from(model.id),
            email,
            password,
            profile,
            model.create_at,
            roles,
        ))
    }

    /// Convert models to domain users, loading all of their roles in one query
    async fn to_domain_many(
        &self,
        models: Vec<users::Model>,
    ) -> Result<Vec<User>, RepositoryError> {
        let ids: Vec<i32> = models.iter().map(|m| m.id).collect();
        let mut roles = self.load_roles(&ids).await?;

        models
            .into_iter()
            .map(|model| {
                let user_roles = roles.remove(&model.id).unwrap_or_default();
                Self::to_domain(model, user_roles)
            })
            .collect()
    }

    /// Convert domain User to SeaORM ActiveModel
    fn to_active_model(user: &User) -> users::ActiveModel {
        users::ActiveModel {
            id: match user.id() {
                Some(id) => Set(id.value()),
                None => Default::default(),
            },
            email: Set(user.email().to_string()),
            password_hash: Set(user.password().hashed().to_string()),
            first_name: Set(user.profile().first_name().to_string()),
            last_name: Set(user.profile().last_name().to_string()),
            age: Set(i32::from(user.profile().age())),
            create_at: Set(user.created_at()),
        }
    }
}

#[async_trait]
impl UserRepository for SeaOrmUserRepository {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        let model = UsersEntity::find_by_id(id.value())
            .one(self.db.as_ref())
            .await
            .map_err(persistence_error)?;

        Ok(self
            .to_domain_many(model.into_iter().collect())
            .await?
            .pop())
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        let model = UsersEntity::find()
            .filter(users::Column::Email.eq(email.as_ref()))
            .one(self.db.as_ref())
            .await
            .map_err(persistence_error)?;

        Ok(self
            .to_domain_many(model.into_iter().collect())
            .await?
            .pop())
    }

    async fn save(&self, user: &mut User) -> Result<(), RepositoryError> {
        let txn = self.db.begin().await.map_err(persistence_error)?;

        let active_model = Self::to_active_model(user);
        let user_id = match user.id() {
            None => {
                active_model
                    .insert(&txn)
                    .await
                    .map_err(persistence_error)?
                    .id
            }
            Some(_) => {
                active_model
                    .update(&txn)
                    .await
                    .map_err(persistence_error)?
                    .id
            }
        };

        Self::save_roles(&txn, user_id, user.roles()).await?;

        txn.commit().await.map_err(persistence_error)?;

        // Only expose the ID once the user is durably stored
        user.set_id(UserId::from(user_id));

        Ok(())
    }

    async fn exists_with_email(&self, email: &Email) -> Result<bool, RepositoryError> {
        let count = UsersEntity::find()
            .filter(users::Column::Email.eq(email.as_ref()))
            .count(self.db.as_ref())
            .await
            .map_err(persistence_error)?;

        Ok(count > 0)
    }

    async fn list(
        &self,
        page: u64,
        rows_per_page: u64,
    ) -> Result<(Vec<User>, u64), RepositoryError> {
        let offset = (page.saturating_sub(1)) * rows_per_page;

        let models = UsersEntity::find()
            .order_by_desc(users::Column::Id)
            .offset(offset)
            .limit(rows_per_page)
            .all(self.db.as_ref())
            .await
            .map_err(persistence_error)?;

        let total = UsersEntity::find()
            .count(self.db.as_ref())
            .await
            .map_err(persistence_error)?;

        Ok((self.to_domain_many(models).await?, total))
    }

    async fn find_roles_by_user_id(&self, id: UserId) -> Result<HashSet<Role>, RepositoryError> {
        let mut roles = self.load_roles(&[id.value()]).await?;
        Ok(roles.remove(&id.value()).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Value};
    use std::collections::BTreeMap;

    fn user_model(id: i32, email: &str) -> users::Model {
        users::Model {
            id,
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            age: 30,
            email: email.to_string(),
            password_hash: "$2b$04$hash".to_string(),
            create_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        }
    }

    fn role_model(id: i32, name: &str) -> roles::Model {
        roles::Model {
            id,
            name: name.to_string(),
        }
    }

    fn user_role(user_id: i32, role_id: i32) -> user_roles::Model {
        user_roles::Model { user_id, role_id }
    }

    fn count_row(count: i64) -> BTreeMap<&'static str, Value> {
        BTreeMap::from([("num_items", Value::BigInt(Some(count)))])
    }

    #[tokio::test]
    async fn test_list_loads_roles_for_all_users_in_one_query() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![
                user_model(2, "b@example.com"),
                user_model(1, "a@example.com"),
            ]])
            .append_query_results([vec![count_row(2)]])
            .append_query_results([vec![
                (user_role(1, 1), Some(role_model(1, "admin"))),
                (user_role(1, 2), Some(role_model(2, "user"))),
                (user_role(2, 2), Some(role_model(2, "user"))),
            ]])
            .into_connection();
        let db = Arc::new(db);
        let repository = SeaOrmUserRepository::new(db.clone());

        let (users, total) = repository.list(1, 10).await.unwrap();

        assert_eq!(total, 2);
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].roles(), &HashSet::from([Role::User]));
        assert_eq!(users[1].roles(), &HashSet::from([Role::Admin, Role::User]));

        drop(repository);
        let log = Arc::try_unwrap(db).unwrap().into_transaction_log();
        assert_eq!(log.len(), 3, "expected users, count and roles queries only");
    }

    #[tokio::test]
    async fn test_save_new_user_inserts_user_and_roles_in_one_transaction() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user_model(7, "new@example.com")]])
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            }])
            .append_query_results([vec![role_model(2, "user")]])
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();
        let db = Arc::new(db);
        let repository = SeaOrmUserRepository::new(db.clone());

        let mut user = User::register(
            Email::try_from("new@example.com").unwrap(),
            "SecurePass123".to_string(),
            "Test".to_string(),
            "User".to_string(),
            30,
        )
        .unwrap();

        repository.save(&mut user).await.unwrap();
        assert_eq!(user.id().map(|id| id.value()), Some(7));

        drop(repository);
        let log = Arc::try_unwrap(db).unwrap().into_transaction_log();
        assert_eq!(
            log.len(),
            1,
            "all statements should run in a single transaction"
        );
        assert_eq!(
            log[0].statements().len(),
            6,
            "begin, insert, delete, select, insert, commit"
        );
    }

    #[test]
    fn test_unexpected_db_error_maps_to_persistence_failure() {
        let error = persistence_error(DbErr::Custom("boom".to_string()));
        assert!(matches!(error, RepositoryError::PersistenceFailure(_)));
    }
}

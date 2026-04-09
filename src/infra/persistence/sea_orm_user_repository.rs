use super::entities::roles::{self, Entity as RolesEntity};
use super::entities::user_roles::{self, Entity as UserRolesEntity};
use super::entities::users::{self, Entity as UsersEntity};
use crate::domain::shared::UserId;
use crate::domain::user::entity::User;
use crate::domain::user::repository::{RepositoryError, UserRepository};
use crate::domain::user::{Email, Password, Role, UserProfile};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

/// SeaORM implementation of UserRepository
pub struct SeaOrmUserRepository {
    db: Arc<sea_orm::DatabaseConnection>,
}

impl SeaOrmUserRepository {
    pub fn new(db: Arc<sea_orm::DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Load roles for a given user ID from the junction table
    async fn load_roles(&self, user_id: i32) -> Result<HashSet<Role>, RepositoryError> {
        let role_models = UserRolesEntity::find()
            .filter(user_roles::Column::UserId.eq(user_id))
            .find_also_related(RolesEntity)
            .all(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::PersistenceFailure(e.to_string()))?;

        let mut roles = HashSet::new();
        for (_user_role, role_opt) in role_models {
            if let Some(role_model) = role_opt {
                if let Ok(role) = Role::from_str(&role_model.name) {
                    roles.insert(role);
                }
            }
        }

        Ok(roles)
    }

    /// Save roles for a user by syncing the junction table
    async fn save_roles(&self, user_id: i32, roles: &HashSet<Role>) -> Result<(), RepositoryError> {
        // Delete existing role assignments
        UserRolesEntity::delete_many()
            .filter(user_roles::Column::UserId.eq(user_id))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::PersistenceFailure(e.to_string()))?;

        // Insert new role assignments
        for role in roles {
            let role_name = role.to_string();

            // Look up the role ID by name
            let role_model = RolesEntity::find()
                .filter(roles::Column::Name.eq(&role_name))
                .one(self.db.as_ref())
                .await
                .map_err(|e| RepositoryError::PersistenceFailure(e.to_string()))?
                .ok_or_else(|| {
                    RepositoryError::PersistenceFailure(format!(
                        "Role '{}' not found in database",
                        role_name
                    ))
                })?;

            let user_role = user_roles::ActiveModel {
                user_id: Set(user_id),
                role_id: Set(role_model.id),
            };

            user_role
                .insert(self.db.as_ref())
                .await
                .map_err(|e| RepositoryError::PersistenceFailure(e.to_string()))?;
        }

        Ok(())
    }

    /// Convert SeaORM model to domain User entity
    async fn to_domain(&self, model: users::Model) -> Result<User, RepositoryError> {
        let user_id = model.id;

        let email = Email::try_from(model.email)
            .map_err(|e| RepositoryError::PersistenceFailure(format!("Invalid email: {}", e)))?;

        let password = Password::from_hash(model.password_hash);

        let profile = UserProfile::new(model.first_name, model.last_name, model.age as u8)
            .map_err(|e| RepositoryError::PersistenceFailure(format!("Invalid profile: {}", e)))?;

        let roles = self.load_roles(user_id).await?;

        Ok(User::reconstitute(
            UserId::from(user_id),
            email,
            password,
            profile,
            model.create_at,
            roles,
        ))
    }

    /// Convert domain User to SeaORM ActiveModel for insert
    fn to_active_model_insert(&self, user: &User) -> users::ActiveModel {
        users::ActiveModel {
            email: Set(user.email().to_string()),
            password_hash: Set(user.password().hashed().to_string()),
            first_name: Set(user.profile().first_name().to_string()),
            last_name: Set(user.profile().last_name().to_string()),
            age: Set(user.profile().age() as i32),
            create_at: Set(user.created_at()),
            ..Default::default()
        }
    }

    /// Convert domain User to SeaORM ActiveModel for update
    fn to_active_model_update(&self, user: &User) -> users::ActiveModel {
        let id = user.id().expect("User must have an ID to update").value();

        users::ActiveModel {
            id: Set(id),
            email: Set(user.email().to_string()),
            password_hash: Set(user.password().hashed().to_string()),
            first_name: Set(user.profile().first_name().to_string()),
            last_name: Set(user.profile().last_name().to_string()),
            age: Set(user.profile().age() as i32),
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
            .map_err(|e| RepositoryError::PersistenceFailure(e.to_string()))?;

        match model {
            Some(m) => Ok(Some(self.to_domain(m).await?)),
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        let model = UsersEntity::find()
            .filter(users::Column::Email.eq(email.to_string()))
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::PersistenceFailure(e.to_string()))?;

        match model {
            Some(m) => Ok(Some(self.to_domain(m).await?)),
            None => Ok(None),
        }
    }

    async fn save(&self, user: &mut User) -> Result<(), RepositoryError> {
        if user.id().is_none() {
            // Insert new user
            let active_model = self.to_active_model_insert(user);
            let inserted = active_model
                .insert(self.db.as_ref())
                .await
                .map_err(|e| RepositoryError::PersistenceFailure(e.to_string()))?;

            // Set the ID on the user entity
            let user_id = inserted.id;
            user.set_id(UserId::from(user_id));

            // Save roles for the new user
            self.save_roles(user_id, user.roles()).await?;
        } else {
            // Update existing user
            let active_model = self.to_active_model_update(user);
            active_model
                .update(self.db.as_ref())
                .await
                .map_err(|e| RepositoryError::PersistenceFailure(e.to_string()))?;

            // Sync roles
            let user_id = user.id().unwrap().value();
            self.save_roles(user_id, user.roles()).await?;
        }

        Ok(())
    }

    async fn exists_with_email(&self, email: &Email) -> Result<bool, RepositoryError> {
        let exists = UsersEntity::find()
            .filter(users::Column::Email.eq(email.to_string()))
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::PersistenceFailure(e.to_string()))?
            .is_some();

        Ok(exists)
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
            .map_err(|e| RepositoryError::PersistenceFailure(e.to_string()))?;

        let total = UsersEntity::find()
            .count(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::PersistenceFailure(e.to_string()))?;

        let mut users = Vec::with_capacity(models.len());
        for model in models {
            users.push(self.to_domain(model).await?);
        }

        Ok((users, total))
    }

    async fn find_roles_by_user_id(&self, id: UserId) -> Result<HashSet<Role>, RepositoryError> {
        self.load_roles(id.value()).await
    }
}

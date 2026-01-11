use super::entities::users::{self, Entity as Users};
use crate::features::shared::UserId;
use crate::features::user::entity::User;
use crate::features::user::repository::{RepositoryError, UserRepository};
use crate::features::user::{Email, Password, UserProfile};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::sync::Arc;

/// SeaORM implementation of UserRepository
pub struct SeaOrmUserRepository {
    db: Arc<sea_orm::DatabaseConnection>,
}

impl SeaOrmUserRepository {
    pub fn new(db: Arc<sea_orm::DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Convert SeaORM model to domain User entity
    fn to_domain(&self, model: users::Model) -> Result<User, RepositoryError> {
        let email = Email::new(model.email)
            .map_err(|e| RepositoryError::DatabaseError(format!("Invalid email: {}", e)))?;

        let password = Password::from_hash(model.password_hash);

        let profile = UserProfile::new(model.first_name, model.last_name, model.age as u8)
            .map_err(|e| RepositoryError::DatabaseError(format!("Invalid profile: {}", e)))?;

        Ok(User::reconstitute(
            UserId::from_i32(model.id),
            email,
            password,
            profile,
            model.create_at,
        ))
    }

    /// Convert domain User to SeaORM ActiveModel for insert
    fn to_active_model_insert(&self, user: &User) -> users::ActiveModel {
        users::ActiveModel {
            email: Set(user.email().as_str().to_string()),
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
            email: Set(user.email().as_str().to_string()),
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
        let model = Users::find_by_id(id.value())
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match model {
            Some(m) => Ok(Some(self.to_domain(m)?)),
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        let model = Users::find()
            .filter(users::Column::Email.eq(email.as_str()))
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match model {
            Some(m) => Ok(Some(self.to_domain(m)?)),
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
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

            // Set the ID on the user entity
            user.set_id(UserId::from_i32(inserted.id));
        } else {
            // Update existing user
            let active_model = self.to_active_model_update(user);
            active_model
                .update(self.db.as_ref())
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    async fn exists_with_email(&self, email: &Email) -> Result<bool, RepositoryError> {
        let exists = Users::find()
            .filter(users::Column::Email.eq(email.as_str()))
            .one(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
            .is_some();

        Ok(exists)
    }

    async fn list(
        &self,
        page: u64,
        rows_per_page: u64,
    ) -> Result<(Vec<User>, u64), RepositoryError> {
        let offset = (page.saturating_sub(1)) * rows_per_page;

        let models = Users::find()
            .order_by_desc(users::Column::Id)
            .offset(offset)
            .limit(rows_per_page)
            .all(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let total = Users::find()
            .count(self.db.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let users: Result<Vec<User>, RepositoryError> =
            models.into_iter().map(|m| self.to_domain(m)).collect();

        Ok((users?, total))
    }
}

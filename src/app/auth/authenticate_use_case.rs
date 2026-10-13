use crate::app::caller_context::CallerContext;
use crate::app::errors::AppResult;
use crate::app::ports::TokenService;
use crate::domain::shared::UserId;
use crate::domain::user::UserRepository;
use std::sync::Arc;

/// AuthenticateUseCase - resolves an access token into the caller's identity and roles
pub struct AuthenticateUseCase {
    user_repository: Arc<dyn UserRepository>,
    token_service: Arc<dyn TokenService>,
}

impl AuthenticateUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        token_service: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            user_repository,
            token_service,
        }
    }

    pub async fn execute(&self, token: &str) -> AppResult<CallerContext> {
        let user_id = self.token_service.verify_token(token).await?;

        // Roles are looked up per request so changes take effect immediately
        let roles = self
            .user_repository
            .find_roles_by_user_id(UserId::from(user_id))
            .await?;

        Ok(CallerContext::new(user_id, roles))
    }
}

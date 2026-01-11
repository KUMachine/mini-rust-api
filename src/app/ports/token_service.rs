use crate::app::errors::ApplicationError;
use async_trait::async_trait;

/// TokenService port - defines the contract for token generation
/// This trait lives in the application layer, implementations are in infrastructure
#[async_trait]
pub trait TokenService: Send + Sync {
    /// Generate an authentication token for a user
    async fn generate_token(
        &self,
        user_id: i32,
        user_email: &str,
    ) -> Result<String, ApplicationError>;
}

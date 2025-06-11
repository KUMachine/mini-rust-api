use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: T,
}

#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub success: bool,
    pub error: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data,
        }
    }
}

impl ApiErrorResponse {
    pub fn error<E: ToString>(message: E) -> Self {
        Self {
            success: false,
            error: message.to_string(),
        }
    }
}

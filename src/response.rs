use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Meta {
    pub count: Option<u64>,
    #[serde(rename = "rowsPerPage")]
    pub rows_per_page: Option<u32>,
    pub page: Option<u32>,
}

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    pub data: T,
}

#[derive(Serialize, ToSchema)]
pub struct ApiErrorResponse {
    pub success: bool,
    pub errors: Vec<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { meta: None, data }
    }

    pub fn with_pagination(data: T, count: u64, rows_per_page: u32, page: u32) -> Self {
        Self {
            meta: Some(Meta {
                count: Some(count),
                rows_per_page: Some(rows_per_page),
                page: Some(page),
            }),
            data,
        }
    }
}

impl ApiErrorResponse {
    pub fn new(errors: Vec<String>) -> Self {
        Self {
            success: false,
            errors,
        }
    }
}

use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PaginationRequest {
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_rows_per_page")]
    #[validate(range(
        min = 1,
        max = 100,
        message = "Rows per page must be between 1 and 100"
    ))]
    pub rows_per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_rows_per_page() -> u32 {
    10
}

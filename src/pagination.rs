use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PaginationRequest {
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_rows_per_page", rename = "rowsPerPage")]
    pub rows_per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_rows_per_page() -> u32 {
    10
}

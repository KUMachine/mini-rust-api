//! API response types following JSON:API specification

use serde::Serialize;
use utoipa::ToSchema;

/// Metadata for paginated responses
#[derive(Serialize, ToSchema)]
pub struct Meta {
    pub count: Option<u64>,
    #[serde(rename = "rowsPerPage")]
    pub rows_per_page: Option<u32>,
    pub page: Option<u32>,
}

/// Standard API response wrapper
#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    pub data: T,
}

/// Source of a JSON:API error
#[derive(Serialize, ToSchema)]
pub struct JsonApiErrorSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
}

/// JSON:API error object
#[derive(Serialize, ToSchema)]
pub struct JsonApiError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<JsonApiErrorSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// JSON:API error response
#[derive(Serialize, ToSchema)]
pub struct ApiErrorResponse {
    pub errors: Vec<JsonApiError>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response without pagination
    pub fn ok(data: T) -> Self {
        Self { meta: None, data }
    }

    /// Create a successful response with pagination metadata
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

impl JsonApiError {
    /// Create a new JSON:API error
    pub fn new(status: u16, code: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: None,
            status: status.to_string(),
            code: Some(code.into()),
            title: title.into(),
            detail: None,
            source: None,
            meta: None,
        }
    }

    /// Add detail to the error
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Add a source pointer to the error
    pub fn with_source_pointer(mut self, pointer: impl Into<String>) -> Self {
        self.source = Some(JsonApiErrorSource {
            pointer: Some(pointer.into()),
            parameter: None,
            header: None,
        });
        self
    }

    /// Add a source parameter to the error
    pub fn with_source_parameter(mut self, parameter: impl Into<String>) -> Self {
        self.source = Some(JsonApiErrorSource {
            pointer: None,
            parameter: Some(parameter.into()),
            header: None,
        });
        self
    }

    /// Add metadata to the error
    pub fn with_meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

impl ApiErrorResponse {
    /// Create an error response from multiple errors
    pub fn new(errors: Vec<JsonApiError>) -> Self {
        Self { errors }
    }

    /// Create an error response from a single error
    pub fn from_single_error(error: JsonApiError) -> Self {
        Self {
            errors: vec![error],
        }
    }
}

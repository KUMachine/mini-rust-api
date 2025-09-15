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
pub struct JsonApiErrorSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
}

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

#[derive(Serialize, ToSchema)]
pub struct ApiErrorResponse {
    pub errors: Vec<JsonApiError>,
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

impl JsonApiError {
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

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_source_pointer(mut self, pointer: impl Into<String>) -> Self {
        self.source = Some(JsonApiErrorSource {
            pointer: Some(pointer.into()),
            parameter: None,
            header: None,
        });
        self
    }

    pub fn with_source_parameter(mut self, parameter: impl Into<String>) -> Self {
        self.source = Some(JsonApiErrorSource {
            pointer: None,
            parameter: Some(parameter.into()),
            header: None,
        });
        self
    }

    pub fn with_meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

impl ApiErrorResponse {
    pub fn new(errors: Vec<JsonApiError>) -> Self {
        Self { errors }
    }

    pub fn from_single_error(error: JsonApiError) -> Self {
        Self {
            errors: vec![error],
        }
    }
}

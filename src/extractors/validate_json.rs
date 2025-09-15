use crate::response::{ApiErrorResponse, JsonApiError};
use axum::{
    extract::{Json, Request, rejection::JsonRejection},
    http::StatusCode,
    response::Response,
};
use axum_core::extract::FromRequest;
use axum_core::response::IntoResponse;
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

pub struct ValidationRejection(Vec<String>);

impl IntoResponse for ValidationRejection {
    fn into_response(self) -> Response {
        let errors: Vec<JsonApiError> = self
            .0
            .into_iter()
            .map(|err| {
                JsonApiError::new(422, "VALIDATION_ERROR", "Validation Failed").with_detail(err)
            })
            .collect();
        let body = Json(ApiErrorResponse::new(errors));
        (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
    }
}

fn json_rejection_to_response(rejection: JsonRejection) -> Response {
    match rejection {
        JsonRejection::JsonDataError(error) => {
            let error = JsonApiError::new(422, "JSON_DATA_ERROR", "JSON Deserialization Failed")
                .with_detail(error.to_string());
            let body = Json(ApiErrorResponse::from_single_error(error));
            (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
        }
        JsonRejection::JsonSyntaxError(_) => {
            let error = JsonApiError::new(400, "JSON_SYNTAX_ERROR", "Invalid JSON Syntax")
                .with_detail("Invalid JSON syntax in request body");
            let body = Json(ApiErrorResponse::from_single_error(error));
            (StatusCode::BAD_REQUEST, body).into_response()
        }
        JsonRejection::MissingJsonContentType(_) => {
            let error =
                JsonApiError::new(415, "MISSING_CONTENT_TYPE", "Missing Content-Type Header")
                    .with_detail(
                        "Missing or invalid Content-Type header. Expected 'application/json'",
                    )
                    .with_source_parameter("Content-Type");
            let body = Json(ApiErrorResponse::from_single_error(error));
            (StatusCode::UNSUPPORTED_MEDIA_TYPE, body).into_response()
        }
        JsonRejection::BytesRejection(_) => {
            let error = JsonApiError::new(400, "REQUEST_BODY_ERROR", "Request Body Error")
                .with_detail("Failed to read request body");
            let body = Json(ApiErrorResponse::from_single_error(error));
            (StatusCode::BAD_REQUEST, body).into_response()
        }
        _ => {
            // Fallback for any future variants
            let error = JsonApiError::new(
                rejection.status().as_u16(),
                "JSON_PROCESSING_ERROR",
                "JSON Processing Error",
            )
            .with_detail("An error occurred while processing JSON data");
            let body = Json(ApiErrorResponse::from_single_error(error));
            (rejection.status(), body).into_response()
        }
    }
}

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => {
                if let Err(errors) = value.validate() {
                    let error_messages: Vec<String> = errors
                        .field_errors()
                        .into_iter()
                        .flat_map(|(field, field_errors)| {
                            let field = field.to_string();
                            field_errors.iter().map(move |error| {
                                if let Some(message) = &error.message {
                                    message.to_string()
                                } else {
                                    format!("{} is invalid", field)
                                }
                            })
                        })
                        .collect();
                    return Err(ValidationRejection(error_messages).into_response());
                }
                Ok(ValidatedJson(value))
            }
            Err(rejection) => Err(json_rejection_to_response(rejection)),
        }
    }
}

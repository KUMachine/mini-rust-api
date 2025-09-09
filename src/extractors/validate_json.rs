use crate::response::ApiErrorResponse;
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
        let body = Json(ApiErrorResponse::new(self.0));
        (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
    }
}

fn json_rejection_to_response(rejection: JsonRejection) -> Response {
    match rejection {
        JsonRejection::JsonDataError(_) => {
            let error_message = "Failed to deserialize JSON into the expected format".to_string();
            let body = Json(ApiErrorResponse::new(vec![error_message]));
            (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
        }
        JsonRejection::JsonSyntaxError(_) => {
            let error_message = "Invalid JSON syntax in request body".to_string();
            let body = Json(ApiErrorResponse::new(vec![error_message]));
            (StatusCode::BAD_REQUEST, body).into_response()
        }
        JsonRejection::MissingJsonContentType(_) => {
            let error_message =
                "Missing or invalid Content-Type header. Expected 'application/json'".to_string();
            let body = Json(ApiErrorResponse::new(vec![error_message]));
            (StatusCode::UNSUPPORTED_MEDIA_TYPE, body).into_response()
        }
        JsonRejection::BytesRejection(_) => {
            let error_message = "Failed to read request body".to_string();
            let body = Json(ApiErrorResponse::new(vec![error_message]));
            (StatusCode::BAD_REQUEST, body).into_response()
        }
        _ => {
            // Fallback for any future variants
            let error_message = "JSON processing error".to_string();
            let body = Json(ApiErrorResponse::new(vec![error_message]));
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

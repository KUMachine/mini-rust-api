use crate::response::ApiErrorResponse;
use axum::{
    extract::{FromRequest, Json, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

#[derive(Debug)]
pub struct ValidationRejection(Vec<String>);

impl IntoResponse for ValidationRejection {
    fn into_response(self) -> Response {
        let body = Json(ApiErrorResponse::new(self.0));
        (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
    }
}

fn json_rejection_to_response(rejection: JsonRejection) -> Response {
    let body = Json(ApiErrorResponse::new(vec![rejection.to_string()]));
    (rejection.status(), body).into_response()
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

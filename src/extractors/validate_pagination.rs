use crate::pagination::PaginationRequest;
use crate::response::ApiErrorResponse;
use axum::{
    extract::{Query, Request, rejection::QueryRejection},
    http::StatusCode,
    response::Response,
};
use axum_core::extract::FromRequest;
use axum_core::response::IntoResponse;
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedPagination<T>(pub T);

pub struct PaginationValidationRejection(Vec<String>);

impl IntoResponse for PaginationValidationRejection {
    fn into_response(self) -> Response {
        let body = axum::Json(ApiErrorResponse::new(self.0));
        (StatusCode::BAD_REQUEST, body).into_response()
    }
}

fn query_rejection_to_response(rejection: QueryRejection) -> Response {
    let body = axum::Json(ApiErrorResponse::new(vec![rejection.to_string()]));
    (rejection.status(), body).into_response()
}

impl<S, T> FromRequest<S> for ValidatedPagination<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Query::<T>::from_request(req, state).await {
            Ok(Query(value)) => {
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
                    return Err(PaginationValidationRejection(error_messages).into_response());
                }
                Ok(ValidatedPagination(value))
            }
            Err(rejection) => Err(query_rejection_to_response(rejection)),
        }
    }
}

// Type alias for pagination query specifically
pub type PaginationQuery = ValidatedPagination<PaginationRequest>;

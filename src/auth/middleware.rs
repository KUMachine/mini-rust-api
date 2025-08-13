use axum::{
    RequestPartsExt,
    extract::{Request, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
use axum_core::extract::FromRequestParts;
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{Validation, decode};

use super::service::get_decoding_key;
use super::types::{AuthError, Claims};
use crate::AppState;

pub async fn auth_middleware(
    State(_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let (mut parts, body) = req.into_parts();

    match Claims::from_request_parts(&mut parts, &_state).await {
        Ok(claims) => {
            // Add the claims to the request extensions so handlers can access them
            req = Request::from_parts(parts, body);
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

// Helper function to get claims from request extensions
pub fn get_claims_from_request(req: &Request) -> Option<&Claims> {
    req.extensions().get::<Claims>()
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        // Decode the user data
        let token_data =
            decode::<Claims>(bearer.token(), get_decoding_key(), &Validation::default())
                .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

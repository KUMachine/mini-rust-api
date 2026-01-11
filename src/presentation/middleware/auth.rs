//! Authentication middleware
//!
//! JWT token validation middleware for protected routes.

use super::super::state::AppState;
use crate::infra::auth::jwt_token_service::{Claims, JwtTokenService};
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

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let (mut parts, body) = req.into_parts();

    match Claims::from_request_parts(&mut parts, &_state).await {
        Ok(claims) => {
            req = Request::from_parts(parts, body);
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let token_data = decode::<Claims>(
            bearer.token(),
            JwtTokenService::get_decoding_key(),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(token_data.claims)
    }
}

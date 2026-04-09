//! Authentication middleware
//!
//! JWT token validation middleware for protected routes.
//! After validating the token, roles are looked up from the database
//! and a CallerContext is inserted into request extensions.

use super::super::state::AppState;
use crate::app::CallerContext;
use crate::domain::shared::UserId;
use crate::infra::auth::jwt_token_service::{Claims, JwtTokenService};
use axum::{
    RequestPartsExt,
    extract::{Request, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{Validation, decode};

/// Authentication middleware that validates JWT tokens and builds CallerContext
///
/// 1. Decodes and validates the JWT token
/// 2. Looks up the user's current roles from the database
/// 3. Inserts a CallerContext into request extensions for downstream handlers
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let (mut parts, body) = req.into_parts();

    // Extract and validate the JWT token
    let claims = extract_claims(&mut parts).await?;

    // Look up current roles from the database
    let roles = state
        .user_repository
        .find_roles_by_user_id(UserId::from(claims.user_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Build CallerContext with fresh roles from DB
    let caller = CallerContext::new(claims.user_id, roles);

    req = Request::from_parts(parts, body);
    req.extensions_mut().insert(caller);
    Ok(next.run(req).await)
}

/// Extract and validate JWT claims from the request
async fn extract_claims(parts: &mut Parts) -> Result<Claims, StatusCode> {
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

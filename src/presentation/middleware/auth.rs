//! Authentication middleware
//!
//! Bearer token authentication for protected routes. The token is resolved
//! into a CallerContext (identity + current roles) which is inserted into
//! request extensions for handler extraction.

use super::super::state::AppState;
use crate::app::ApplicationError;
use axum::{
    RequestPartsExt,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

/// Authentication middleware that validates bearer tokens and builds CallerContext
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApplicationError> {
    let (mut parts, body) = req.into_parts();

    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| ApplicationError::Unauthorized)?;

    let caller = state.authenticate_use_case.execute(bearer.token()).await?;

    let mut req = Request::from_parts(parts, body);
    req.extensions_mut().insert(caller);
    Ok(next.run(req).await)
}

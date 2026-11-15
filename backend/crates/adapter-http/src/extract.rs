use application::security::Principal;
use application::AppError;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use crate::error::ApiError;
use crate::AppState;

/// Extracts and verifies the `Authorization: Bearer <jwt>` header.
/// Only authentication happens here; authorization is the use case's job.
pub struct Auth(pub Principal);

impl FromRequestParts<AppState> for Auth {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthenticated)?;
        Ok(Auth(state.auth.authenticate(token.trim())?))
    }
}

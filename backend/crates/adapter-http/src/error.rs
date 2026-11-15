use application::AppError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use domain::DomainError;
use serde_json::json;

pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        Self(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self.0 {
            AppError::Unauthenticated => (StatusCode::UNAUTHORIZED, "unauthenticated"),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            AppError::NotFound(_) | AppError::Domain(DomainError::UnknownMetric(_)) => {
                (StatusCode::NOT_FOUND, "not_found")
            }
            AppError::Domain(_) | AppError::Validation(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "validation_error")
            }
            AppError::Infrastructure(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };
        let message = match &self.0 {
            // Never leak infrastructure details to clients.
            AppError::Infrastructure(_) => "internal server error".to_owned(),
            other => other.to_string(),
        };
        (status, Json(json!({ "error": code, "message": message }))).into_response()
    }
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;

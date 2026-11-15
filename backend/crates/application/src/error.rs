use domain::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("authentication required")]
    Unauthenticated,
    #[error("invalid username or password")]
    InvalidCredentials,
    #[error("access to this resource is not allowed")]
    Forbidden,
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("invalid request: {0}")]
    Validation(String),
    #[error("infrastructure failure: {0}")]
    Infrastructure(String),
}

/// Error type returned by outbound adapters; details stay in the message.
#[derive(Debug, Error)]
#[error("{0}")]
pub struct InfraError(pub String);

impl InfraError {
    pub fn new(e: impl std::fmt::Display) -> Self {
        Self(e.to_string())
    }
}

impl From<InfraError> for AppError {
    fn from(e: InfraError) -> Self {
        AppError::Infrastructure(e.0)
    }
}

pub type AppResult<T> = Result<T, AppError>;

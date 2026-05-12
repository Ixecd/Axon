use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid identity: {0}")]
    InvalidIdentity(String),

    #[error("Invalid disbursement: {0}")]
    InvalidDisburse(String),

    #[error("No route available for this disbursement")]
    NoRoute,

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match &self {
            AppError::InvalidIdentity(m) => (StatusCode::FORBIDDEN, m.clone()),
            AppError::InvalidDisburse(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::NoRoute => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(ErrorBody { error: msg })).into_response()
    }
}

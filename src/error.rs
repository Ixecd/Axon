use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid transfer: {0}")]
    InvalidTransfer(String),

    #[error("No route available for this transfer")]
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
            AppError::InvalidTransfer(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::NoRoute => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(ErrorBody { error: msg })).into_response()
    }
}

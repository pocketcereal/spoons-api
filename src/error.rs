//! Application error types and handling.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Application-level errors.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Error response body for JSON responses.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Server(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Internal(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        tracing::error!(error = %self, "Request failed");

        let body = Json(ErrorResponse {
            error: status.canonical_reason().unwrap_or("Error").to_string(),
            message: Some(error_message),
        });

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

//! Application error types and handling.

use async_graphql::{Error as GraphQLError, ErrorExtensions};
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

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl ErrorExtensions for AppError {
    fn extend(&self) -> GraphQLError {
        GraphQLError::new(format!("{}", self)).extend_with(|_err, e| match self {
            AppError::Config(_) => e.set("code", "CONFIG_ERROR"),
            AppError::Server(_) => e.set("code", "SERVER_ERROR"),
            AppError::Database(_) => e.set("code", "DATABASE_ERROR"),
            AppError::NotFound(_) => e.set("code", "NOT_FOUND"),
            AppError::Unauthorized(_) => e.set("code", "UNAUTHORIZED"),
            AppError::RateLimited => e.set("code", "RATE_LIMITED"),
            AppError::Internal(_) => e.set("code", "INTERNAL_ERROR"),
        })
    }
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
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "Rate limited".to_string()),
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

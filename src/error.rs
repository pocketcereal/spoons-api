use async_graphql::{Error as GraphQLError, ErrorExtensions};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
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

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Feature disabled: {0}")]
    FeatureDisabled(String),

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
        let message = match self {
            AppError::Config(_)
            | AppError::Server(_)
            | AppError::Database(_)
            | AppError::Internal(_) => "Internal server error".to_string(),
            other => format!("{}", other),
        };
        GraphQLError::new(message).extend_with(|_err, e| match self {
            AppError::Config(_) => e.set("code", "CONFIG_ERROR"),
            AppError::Server(_) => e.set("code", "SERVER_ERROR"),
            AppError::Database(_) => e.set("code", "DATABASE_ERROR"),
            AppError::NotFound(_) => e.set("code", "NOT_FOUND"),
            AppError::Unauthorized(_) => e.set("code", "UNAUTHORIZED"),
            AppError::RateLimited => e.set("code", "RATE_LIMITED"),
            AppError::InvalidInput(_) => e.set("code", "INVALID_INPUT"),
            AppError::FeatureDisabled(_) => e.set("code", "FEATURE_DISABLED"),
            AppError::Internal(_) => e.set("code", "INTERNAL_ERROR"),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "Rate limited".to_string()),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::FeatureDisabled(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            AppError::Config(_) | AppError::Server(_) | AppError::Database(_) | AppError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        if status.is_server_error() {
            tracing::error!(error = %self, "Request failed");
        } else {
            tracing::warn!(error = %self, "Request failed");
        }

        let body = Json(ErrorResponse {
            error: status.canonical_reason().unwrap_or("Error").to_string(),
            message: Some(error_message),
        });

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

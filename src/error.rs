use askama::Error as TemplateError;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("fetch failed: {0}")]
    Fetch(String),
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("template error")]
    Template(#[from] TemplateError),
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) | AppError::Fetch(_) => StatusCode::BAD_REQUEST,
            AppError::Database(_) | AppError::Template(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        tracing::warn!(error = %self, %status, "request failed");
        (status, Html(format!("<h1>{status}</h1><p>{self}</p>"))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

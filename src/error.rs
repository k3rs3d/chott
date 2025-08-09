use actix_web::{HttpResponse, ResponseError};
use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Template error: {0}")]
    TemplateError(#[from] tera::Error),

    #[error("Page not found: {0}")]
    PageNotFound(String),

    #[error("Session error")]
    SessionError(String),

    #[error("Environment error: {0}")]
    EnvironmentError(String),

    #[error("DateTime error: {0}")]
    DateTimeError(#[from] SystemTimeError),

    #[error("Mutex error: {0}")]
    MutexError(String),

    #[error("Other: {0}")]
    OtherError(String),
}

impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError::OtherError(error)
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, msg) = match self {
            AppError::PageNotFound(path) => (
                actix_web::http::StatusCode::NOT_FOUND,
                format!("Page not found: {path}"),
            ),
            AppError::TemplateError(e) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Rendering error: {e}"),
            ),
            other => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                other.to_string(),
            ),
        };

        // placeholder HTML error page
        let body = format!(
            "<html><head><title>Chott Error</title></head>\
            <body><h1>Oops!</h1><p><center>{msg}</center></p></body></html>"
        );
        HttpResponse::build(status)
            .content_type("text/html; charset=utf-8")
            .body(body)
    }
}

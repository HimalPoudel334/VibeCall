use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use actix_web::{HttpResponse, ResponseError, Result as ActixResult, http::StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            message: None,
            error: Some(error),
        }
    }

    pub fn not_found(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
            error: None,
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    Validation(String),

    InternalServerError(String),

    NotFound(String),

    BadRequest(String),

    Unauthorized(String),

    Database(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl Error for AppError {}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Validation(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::InternalServerError(_) | AppError::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        let body = match self {
            AppError::NotFound(msg) => ApiResponse::<()>::not_found(msg.clone()),
            AppError::Validation(msg)
            | AppError::BadRequest(msg)
            | AppError::Unauthorized(msg)
            | AppError::Database(msg)
            | AppError::InternalServerError(msg) => ApiResponse::<()>::error(msg.clone()),
        };

        HttpResponse::build(self.status_code()).json(body)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::Database(error.to_string())
    }
}

pub fn respond_ok<T: serde::Serialize>(data: T) -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AppError>();
    }
}

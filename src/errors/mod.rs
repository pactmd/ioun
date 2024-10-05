use axum::{extract::FromRequest, http::StatusCode, response::{IntoResponse, Response}};
use serde_json::json;
use thiserror::Error;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("SqlxError: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("JsonRejection: {0}")]
    JsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[error("HashError: {0}")]
    HashError(#[from] argon2::password_hash::Error),
    #[error("NotFound")]
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::SqlxError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JsonRejection(ref rejection) => rejection.status(),
            Self::HashError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
        };

        (
            status,
            Json(json!({
                "error": self.to_string()
            }))
        ).into_response()
    }
}
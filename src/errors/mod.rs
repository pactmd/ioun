use axum::{
    extract::FromRequest,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

pub type AppResult<T> = std::result::Result<Json<T>, AppError>;

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
#[error("app error: {0}")]
pub enum AppError {
    SqlxError(#[from] sqlx::Error),
    JsonRejection(#[from] axum::extract::rejection::JsonRejection),
    HashError(#[from] argon2::password_hash::Error),
    #[error("ValidationError: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    #[error("NotFound")]
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("{}", &self);

        let status = match self {
            Self::SqlxError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JsonRejection(ref rejection) => rejection.status(),
            Self::HashError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ValidationError(..) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
        };

        (
            status,
            Json(json!({
                "error": self.to_string()
            })),
        )
            .into_response()
    }
}

use axum::{extract::FromRequest, http::StatusCode, response::{IntoResponse, Response}, Router};
use serde_json::{json, Value};
use thiserror::Error;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::AppConfig;

pub mod auth;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn router() -> Router<AppConfig> {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(root))
        .nest("/auth", auth::router())
        .fallback(|| async {ApiError::NotFound})
        .split_for_parts();

    router.merge(SwaggerUi::new("/docs")
        .url("/docs/openapi.json", api))
}

#[utoipa::path(get, path = "/")]
async fn root() -> Json<Value> {
    Json(json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct Json<T>(T);

impl<T> IntoResponse for Json<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("SqlxError: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("JsonRejection: {0}")]
    JsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[error("NotFound")]
    NotFound,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::SqlxError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JsonRejection(ref rejection) => rejection.status(),
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
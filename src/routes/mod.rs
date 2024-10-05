use axum::Router;
use serde_json::{json, Value};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::{errors::{AppError, Json}, AppConfig};

pub mod auth;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn router() -> Router<AppConfig> {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(root))
        .nest("/auth", auth::router())
        .fallback(|| async {AppError::NotFound})
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
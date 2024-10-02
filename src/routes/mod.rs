use axum::{http::StatusCode, response::IntoResponse, Json, Router};
use serde_json::{json, Value};
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
        .fallback(not_found)
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

// TODO: use error struct here
async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "the requested route does not exist")
}
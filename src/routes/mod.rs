use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::{json, Value};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// TODO: once more stable use: https://crates.io/crates/utoipa-axum
#[derive(OpenApi)]
#[openapi(
    paths(get_root)
)]
struct ApiDoc;

pub fn router() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(get_root))
        .fallback(not_found)
}

#[utoipa::path(
    get,
    path = "/",
)]
async fn get_root() -> Json<Value> {
    Json(json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// TODO: use error struct here
async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "the requested route does not exist")
}
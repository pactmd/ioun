use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::{json, Value};

use crate::AppConfig;

pub mod auth;

pub fn router() -> Router<AppConfig> {
    Router::new()
        .route("/", get(root))
        .nest("/auth", auth::router())
        .fallback(not_found)
}

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
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::{json, Value};

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_root))
        .fallback(not_found)
}

async fn get_root() -> Json<Value> {
    Json(json!({
        "name": "ioun",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// TODO: use error struct here
async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "the requested route does not exist")
}
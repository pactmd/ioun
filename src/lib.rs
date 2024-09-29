use axum::{extract::MatchedPath, http::Request, Router};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing::info_span;

mod database;
mod routes;

pub struct AppConfig {
    pub postgres_pool: PgPool,
}

impl AppConfig {
    pub async fn new() -> Self {
        let postgres_pool = database::postgres::connect()
            .await
            .expect("Postgres connection failed");

        AppConfig {
            postgres_pool
        }
    }

    pub fn service(&self) -> Router {
        Router::new()
            .with_state(self.postgres_pool.clone())
            .merge(routes::router())
            .layer(TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                    )
                })
            )
    }
}
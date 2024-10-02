use axum::{extract::MatchedPath, http::Request, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::trace::TraceLayer;
use tracing::info_span;

mod models;
mod routes;

#[derive(Clone)]
pub struct AppConfig {
    pub postgres_pool: PgPool,
}

impl AppConfig {
    pub async fn new() -> Self {
        tracing::info!("Initializing postgres connection");

        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL not set");

        let postgres_pool = PgPoolOptions::new().connect(&database_url)
            .await
            .expect("Postgres connection failed");

        AppConfig {
            postgres_pool
        }
    }

    pub async fn run_postgres_migrations(&self) {
        tracing::info!("Running postgres migrations");

        sqlx::migrate!().run(&self.postgres_pool)
            .await
            .expect("Postgres migrations failed");
    }

    pub fn service(&self) -> Router {
        Router::new()
            // Add routes
            .merge(routes::router())
            // Put config into server state
            .with_state(self.clone())
            // Add tracing middleware
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
use axum::{extract::MatchedPath, http::Request, Router};
use derive_builder::Builder;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::trace::TraceLayer;
use tracing::info_span;

mod errors;
mod models;
mod routes;

#[derive(Builder, Clone)]
pub struct AppConfig {
    pub url: String,
    pub postgres_pool: PgPool,
}

impl AppConfig {
    pub async fn new() -> Self {
        // TODO: move into async closure
        tracing::info!("Initializing postgres connection to {}", std::env::var("DATABASE_URL").expect("DATABASE_URL not set"));

        AppConfigBuilder::default()
            .url(std::env::var("URL").expect("URL not set"))
            // TODO: once https://github.com/rust-lang/rust/pull/132706 is merged use async closure
            // .postgres_pool(async || {
            //     tracing::info!("Initializing postgres connection");
            //
            //     PgPoolOptions::new()
            //         .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not set"))
            //         .await
            //         .expect("Postgres connection failed")
            // })
            .postgres_pool(
                PgPoolOptions::new()
                    .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not set"))
                    .await
                    .expect("Postgres connection failed")
            )
            .build().expect("Could not build app config")
    }

    pub async fn run_postgres_migrations(&self) {
        tracing::info!("Checking for postgres migrations");

        sqlx::migrate!()
            .run(&self.postgres_pool)
            .await
            .expect("Postgres migrations failed");
    }

    pub fn router(&self) -> Router {
        Router::new()
            // Add routes
            .merge(routes::router())
            // Put config into server state
            .with_state(self.clone())
            // Add tracing middleware
            .layer(
                TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                    )
                }),
            )
    }
}
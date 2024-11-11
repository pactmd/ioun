use axum::{extract::MatchedPath, http::Request, Router};
use derive_builder::Builder;
use sqlx::postgres::PgPoolOptions;
use deadpool_redis::{Config, Runtime};
use tower_http::trace::TraceLayer;
use tracing::info_span;

mod errors;
mod models;
mod routes;

#[derive(Builder, Clone)]
pub struct AppConfig {
    #[builder(setter(into))]
    pub url: String,
    pub postgres_pool: sqlx::PgPool,
    pub redis_pool: deadpool_redis::Pool,
    #[builder(setter(into))]
    session_expire: i64,
}

impl AppConfig {
    pub async fn new() -> Self {
        // Read environment variables from .env file if present
        dotenvy::dotenv().ok();

        // TODO: move into async closure
        tracing::info!(
            "Initializing postgres connection to {}",
            std::env::var("DATABASE_URL").expect("DATABASE_URL not set")
        );

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
            .postgres_pool({
                let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
                tracing::info!("Initializing postgres connection to {}", database_url);

                PgPoolOptions::new()
                    .connect(&database_url)
                    .await
                    .expect("Postgres connection failed")
            })
            .redis_pool({
                let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL not set");
                tracing::info!("Initializing redis connection to: {}", redis_url);

                Config::from_url(redis_url)
                    .builder()
                    .expect("Error building redis pool")
                    .runtime(Runtime::Tokio1)
                    .build()
                    .expect("Redis connection failed")
            })
            .session_expire({
                let session_expire_string = std::env::var("SESSION_EXPIRE").expect("SESSION_EXPIRE not set");
                tracing::info!("Setting session to expire in: {} seconds", session_expire_string);

                session_expire_string.parse::<i64>().expect("Could not parse SESSION_EXPIRE")
            })
            .build()
            .expect("Could not build app config")
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

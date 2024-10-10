use axum::{extract::MatchedPath, http::Request, Router};
use deadpool_redis::{Config, Runtime};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing::info_span;

mod errors;
mod models;
mod routes;

#[derive(Clone)]
pub struct AppConfig {
    pub postgres_pool: sqlx::PgPool,
    pub redis_pool: deadpool_redis::Pool,
    session_expire: i64,
}

impl AppConfig {
    pub async fn new() -> Self {
        // Read environment variables from .env file if present
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
        tracing::info!("Initializing postgres connection to: {}", database_url);
        let postgres_pool = PgPoolOptions::new()
            .connect(&database_url)
            .await
            .expect("Postgres connection failed");

        tracing::info!("Running postgres migrations");
        sqlx::migrate!()
            .run(&postgres_pool)
            .await
            .expect("Postgres migrations failed");

        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL not set");
        tracing::info!("Initializing redis connection to: {}", redis_url);
        let redis_pool = Config::from_url(redis_url)
            .builder()
            .expect("Error building redis pool")
            .runtime(Runtime::Tokio1)
            .build()
            .expect("Redis connection failed");

        let session_expire_string = std::env::var("SESSION_EXPIRE").expect("SESSION_EXPIRE not set");
        tracing::info!("Setting sessions to expire in: {} seconds", session_expire_string);
        let session_expire = session_expire_string.parse::<i64>().expect("Could not parse SESSION_EXPIRE");

        AppConfig {
            postgres_pool,
            redis_pool,
            session_expire,
        }
    }

    pub fn service(&self) -> Router {
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

use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    tracing::info!("Initializing database connection");
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL not set");

    PgPoolOptions::new().connect(&database_url).await
}
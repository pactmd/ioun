use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Read environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create app config
    let app_config = ioun::AppConfig::new().await;

    // Run database migrations
    app_config.run_postgres_migrations().await;

    // Create and bind TCP listener
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Could not create TcpListener");

    // Serve the application
    axum::serve(listener, app_config.service())
        .await
        .expect("Could not serve the application");
}

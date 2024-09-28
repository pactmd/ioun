use axum::Router;

mod database;
mod routes;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let postgres_pool = database::postgres::connect()
        .await
        .expect("Postgres connection failed");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    let app = Router::new()
        .with_state(postgres_pool)
        .merge(routes::router());

    axum::serve(listener, app).await.unwrap();
}
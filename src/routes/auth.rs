use axum::{extract::State, routing::post, Json, Router};

use crate::{models::account::{Account, AccountBody, AccountCredentials}, AppConfig};

pub fn router() -> Router<AppConfig> {
    Router::new()
        .route("/signup", post(signup))
}

async fn signup(
    State(app_config): State<AppConfig>,
    Json(req): Json<AccountBody<AccountCredentials>>,
) -> Json<Account> {
    // TODO: issue session

    let mut transaction = app_config.postgres_pool.begin()
        .await
        .unwrap();

    // Hash password
    let hashed_credentials = req.account.hash_password();

    // Insert into database
    let result = Account::insert(
        &hashed_credentials,
        &mut transaction
    ).await.unwrap();

    transaction.commit().await.unwrap();

    Json(result)
}
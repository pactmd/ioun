use argon2::{password_hash::{rand_core, PasswordHasher, SaltString}, Argon2};
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
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut rand_core::OsRng);

    let password_hash = argon2
        .hash_password(req.account.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let hashed_account = AccountCredentials {
        email: req.account.email,
        password: password_hash
    };

    let mut transaction = app_config.postgres_pool.begin()
        .await
        .unwrap();

    let result = Account::insert(&hashed_account, &mut transaction)
        .await
        .unwrap();

    transaction.commit().await.unwrap();

    Json(result)
}
use axum::extract::State;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    errors::{AppError, Json},
    models::account::{Account, AccountBody, AccountCredentials},
    AppConfig,
};

pub fn router() -> OpenApiRouter<AppConfig> {
    OpenApiRouter::new().routes(routes!(signup))
}

#[utoipa::path(post, path = "/signup")]
async fn signup(
    State(app_config): State<AppConfig>,
    Json(req): Json<AccountBody<AccountCredentials>>,
) -> Result<Json<Account>, AppError> {
    // TODO: issue session

    // Hash password
    let hashed_credentials = req.account.hash_password()?;

    let mut transaction = app_config.postgres_pool.begin().await?;

    // Insert into database
    let result = Account::insert(&hashed_credentials, &mut transaction).await?;

    transaction.commit().await?;

    Ok(Json(result))
}

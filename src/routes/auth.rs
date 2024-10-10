use axum::extract::State;
use serde_json::{json, Value};
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::{
    errors::{AppResult, Json},
    models::{account::{Account, AccountBody, AccountCredentials}, session::{Scope, Session}},
    AppConfig,
};

pub fn router() -> OpenApiRouter<AppConfig> {
    OpenApiRouter::new().routes(routes!(signup))
}

#[utoipa::path(post, path = "/signup")]
async fn signup(
    State(app_config): State<AppConfig>,
    Json(req): Json<AccountBody<AccountCredentials>>,
) -> AppResult<Value> {
    // Validate credentials
    req.account.validate()?;

    // Insert into database
    let mut transaction = app_config.postgres_pool.begin().await?;
    let account = Account::insert(&req.account, &mut transaction).await?;
    transaction.commit().await?;

    // Issue Session
    let session = Session {
        user_id: account.id,
        scopes: Scope::all(),
    };
    let token = Session::insert(
        &session,
        app_config.session_expire,
        &mut app_config.redis_pool.get().await?
    ).await?;

    Ok(Json(json!({
        "token": token,
    })))
}

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::State;
use serde_json::{json, Value};
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::{
    errors::{AppError, AppResult, Json},
    models::{account::{Account, AccountBody, AccountCredentials}, session::{Scope, Session}, Unique},
    AppConfig,
};

pub fn router() -> OpenApiRouter<AppConfig> {
    OpenApiRouter::new()
        .routes(routes!(signup))
        .routes(routes!(signin))
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

#[utoipa::path(post, path = "(signin")]
async fn signin(
    State(app_config): State<AppConfig>,
    Json(req): Json<AccountBody<AccountCredentials>>,
) -> AppResult<Value> {
    // Validate credentials
    req.account.validate()?;

    // Get user with provided email
    let mut transaction = app_config.postgres_pool.begin().await?;
    let account = Account::get(Unique::Email(req.account.email), &mut transaction).await?;
    transaction.commit().await?;

    let token = match account {
        Some(account) => {
            // Hash provided password and compare
            let password_hash = PasswordHash::new(&account.password_hash)?;
            Argon2::default().verify_password(req.account.password.as_bytes(), &password_hash)?;

            let session = Session {
                user_id: account.id,
                scopes: Scope::all(),
            };
            Session::insert(
                &session,
                app_config.session_expire,
                &mut app_config.redis_pool.get().await?
            ).await?
        },
        _ => {
            return Err(AppError::DoesNotExist("user".to_string()))
        }
    };

    Ok(Json(json!({
        "token": token,
    })))
}

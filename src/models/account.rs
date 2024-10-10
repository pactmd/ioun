use argon2::{
    password_hash::{rand_core, PasswordHasher, SaltString},
    Argon2,
};
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;

use super::Unique;

#[derive(Deserialize, ToSchema)]
pub struct AccountBody<T: ToSchema> {
    pub account: T,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct AccountCredentials {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 32))]
    pub password: String,
}

impl std::fmt::Debug for AccountCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountCredentials")
            .field("email", &self.email)
            .field("password", &"*****")
            .finish()
    }
}

pub struct Account {
    pub id: Uuid,
    email: String,
    #[allow(dead_code)]
    pub password_hash: String,
    username: Option<String>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl Account {  
    pub async fn insert(
        command: &AccountCredentials,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Self, AppError> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut rand_core::OsRng);

        let account = sqlx::query_as!(
            Self,
            r#"
            INSERT INTO account (
                email, password_hash
            )
            VALUES (
                $1, $2
            )
            RETURNING
                id,
                email,
                password_hash,
                username AS "username?",
                created_at,
                updated_at
            "#,
            command.email,
            argon2.hash_password(command.password.as_bytes(), &salt)?.to_string(),
        )
        .fetch_one(&mut **transaction)
        .await?;

        Ok(account)
    }

    pub async fn get(
        unique: Unique,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT
                id,
                email,
                password_hash,
                username AS "username?",
                created_at,
                updated_at
            FROM account
            WHERE $1 = $2
            "#,
            unique.key(),
            unique.value(),
        )
        .fetch_optional(&mut **transaction)
        .await
    }
}

impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("id", &self.id)
            .field("email", &self.email)
            .field("password_hash", &"*****")
            .field("username", &self.username)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

impl Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Account", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("email", &self.email)?;
        state.serialize_field("password_hash", &"*****")?;
        state.serialize_field("username", &self.username)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("updated_at", &self.updated_at)?;
        state.end()
    }
}

use argon2::{password_hash::{rand_core, PasswordHasher, SaltString}, Argon2};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
#[aliases(AccountBodyCredentials = AccountBody<AccountCredentials>)]
pub struct AccountBody<T> {
    pub account: T,
}

#[derive(Deserialize, ToSchema)]
pub struct AccountCredentials {
    pub email: String,
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

impl AccountCredentials {
    pub fn hash_password(mut self) -> Self {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut rand_core::OsRng);

        self.password = argon2
            .hash_password(self.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        self
    }
}

#[derive(Serialize)]
pub struct Account {
    id: Uuid,
    email: String,
    password_hash: String,
    username: Option<String>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
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

impl Account {
    pub async fn insert(
        command: &AccountCredentials,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
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
            command.password,
        ).fetch_one(&mut **transaction).await
    }
}
use std::collections::HashSet;

use argon2::{password_hash::{rand_core, PasswordHasher, SaltString}, Argon2};
use deadpool_redis::{redis, Connection};
use redis_macros::FromRedisValue;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;

#[derive(Serialize, Deserialize, FromRedisValue)]
pub struct Session {
    pub user_id: Uuid,
    pub scopes: HashSet<Scope>
}

impl Session {
    pub async fn insert(
        command: &Self,
        expire: i64,
        conn: &mut Connection,
    ) -> Result<Uuid, AppError> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut rand_core::OsRng);

        let token = Uuid::new_v4();
        let token_hash = argon2.hash_password(token.as_bytes(), &salt)?.to_string();

        redis::pipe()
            .atomic()
            .json_set(&token_hash, "$", command)?
            .expire(&token_hash, expire)
            .query_async(conn)
            .await?;

        Ok(token)
    }

    pub async fn get(
        token: String,
        expire: i64,
        conn: &mut Connection,
    ) -> Result<Session, AppError> {
        let session = redis::pipe()
            .atomic()
            .json_get(&token, "$")?
            .expire(&token, expire)
            .ignore()
            .query_async(conn)
            .await?;

        Ok(session)
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Scope {
    Account(Permission),
}

impl Scope {
    pub fn all() -> HashSet<Scope> {
        HashSet::from([
            Scope::Account(Permission::Write)
        ])
    }
}

#[derive(Default, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Permission {
    #[default]
    None,
    Read,
    Write,
}
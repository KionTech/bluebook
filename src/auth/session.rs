//! Sign-ins: a random token in the `bluebook-session` cookie (HttpOnly,
//! SameSite=Lax, Secure on HTTPS), stored as its SHA-256 in `sessions`,
//! good for `SESSION_LIFETIME_DAYS` (400 by default) from the login, so
//! pilots stay signed in. A guest gets a session only when something has
//! to be kept for later (the page to return to after the login, a flash
//! message).
//!
//! `data` holds the active character (`active_character_id`), the account
//! a login adds its character to (`add_to_account`), the page to return
//! to (`intended`) and the flash message of the next page (`flash`).

use axum::http::{HeaderMap, header};
use axum_extra::extract::cookie::{Cookie, SameSite};
use rand::RngExt;
use rand::distr::Alphanumeric;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

pub const SESSION_COOKIE: &str = "bluebook-session";

/// The stored form of a session token.
pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// `Str::random($length)`: letters and digits from a CSPRNG.
pub fn random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// A session as one request sees it.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Session {
    /// The SHA-256 of the cookie's token.
    pub id: String,
    pub user_id: Option<i64>,
    pub data: sqlx::types::Json<Map<String, Value>>,
}

impl Session {
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key).and_then(Value::as_i64)
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(Value::as_str)
    }
}

/// A new session; returns the cookie's token.
pub async fn start(
    pool: &PgPool,
    user_id: Option<i64>,
    data: Map<String, Value>,
    now: OffsetDateTime,
) -> sqlx::Result<String> {
    let token = random_string(40);
    sqlx::query(
        "insert into sessions (id, user_id, data, last_activity, created_at)
         values ($1, $2, $3, $4, $4)",
    )
    .bind(hash_token(&token))
    .bind(user_id)
    .bind(sqlx::types::Json(data))
    .bind(now)
    .execute(pool)
    .await?;
    Ok(token)
}

/// The live session of a cookie token, younger than the lifetime.
pub async fn resolve(
    pool: &PgPool,
    token: &str,
    now: OffsetDateTime,
    lifetime_days: i64,
) -> sqlx::Result<Option<Session>> {
    sqlx::query_as(
        "update sessions set last_activity = $2
         where id = $1 and created_at > $2 - make_interval(days => $3::int)
         returning id, user_id, data",
    )
    .bind(hash_token(token))
    .bind(now)
    .bind(lifetime_days as i32)
    .fetch_optional(pool)
    .await
}

pub async fn end(pool: &PgPool, id: &str) -> sqlx::Result<()> {
    sqlx::query("delete from sessions where id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Sets one value of a session's data.
pub async fn put(pool: &PgPool, id: &str, key: &str, value: Value) -> sqlx::Result<()> {
    sqlx::query("update sessions set data = jsonb_set(data, array[$2], $3) where id = $1")
        .bind(id)
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    Ok(())
}

/// Removes one value of a session's data.
pub async fn forget(pool: &PgPool, id: &str, key: &str) -> sqlx::Result<()> {
    sqlx::query("update sessions set data = data - $2 where id = $1")
        .bind(id)
        .bind(key)
        .execute(pool)
        .await?;
    Ok(())
}

/// Reads and removes one value (the flash message, the intended URL).
pub async fn pull(pool: &PgPool, id: &str, key: &str) -> sqlx::Result<Option<Value>> {
    let value: Option<Option<Value>> = sqlx::query_scalar(
        "update sessions s set data = s.data - $2
         from (select data -> $2 as value from sessions where id = $1 for update) old
         where s.id = $1
         returning old.value",
    )
    .bind(id)
    .bind(key)
    .fetch_optional(pool)
    .await?;
    Ok(value.flatten())
}

/// Deletes the sessions past their lifetime; returns how many.
pub async fn delete_expired(
    pool: &PgPool,
    now: OffsetDateTime,
    lifetime_days: i64,
) -> sqlx::Result<u64> {
    let result =
        sqlx::query("delete from sessions where created_at <= $1 - make_interval(days => $2::int)")
            .bind(now)
            .bind(lifetime_days as i32)
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
}

/// The session cookie for a token.
pub fn cookie(token: String, lifetime_days: i64, secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(Duration::days(lifetime_days))
        .build()
}

/// The cookie that makes the browser forget the session.
pub fn removal(secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, String::new()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(Duration::ZERO)
        .build()
}

/// A cookie's value in a request's `Cookie` headers.
pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| Cookie::parse(pair.trim().to_owned()).ok())
        .find(|cookie| cookie.name() == name)
        .map(|cookie| cookie.value().to_owned())
}

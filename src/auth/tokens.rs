//! The stored EVE SSO tokens: the character's newest token carrying the scope,
//! refreshed first when it expires within five minutes (the rotated refresh
//! token stored).
//!
//! A token is deleted only when it is certainly dead: EVE answers the
//! refresh with `invalid_grant`, or ESI answers 401 right after a fresh
//! refresh (`esi.rs`). Any other failure keeps the token and marks it
//! failing (`failing_since`, the first failure in a row, and
//! `last_failure`); the next success clears the mark. A token failing for
//! `LONG_FAILING` counts as lapsed for the warnings and the token mail.

use std::fmt;

use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

use super::sso::{RefreshError, SsoClient};

/// A token counts as expired this long before its `expires_at`.
const EXPIRY_BUFFER: Duration = Duration::minutes(5);

/// How long a token may keep failing before its character is asked to log
/// in again.
pub const LONG_FAILING: Duration = Duration::days(3);

/// SQL: the token `t` is usable, i.e. not failing for `LONG_FAILING`
/// (`$now` is the bind of the current time).
pub fn usable_sql(now: &str) -> String {
    format!("(t.failing_since is null or t.failing_since > {now} - interval '3 days')")
}

/// A usable access token and the row behind it.
#[derive(Debug, Clone)]
pub struct AccessToken {
    pub token_id: i64,
    pub access_token: String,
    /// Refreshed just now, for this request.
    pub refreshed: bool,
}

#[derive(Debug)]
pub enum TokenError {
    /// EVE said the refresh token is dead; the stored token has been
    /// deleted.
    Dead(String),
    /// The refresh failed for now; the token is kept and marked failing.
    Transient(String),
    Db(sqlx::Error),
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenError::Dead(reason) => write!(f, "token refresh rejected: {reason}"),
            TokenError::Transient(reason) => write!(f, "{reason}"),
            TokenError::Db(error) => write!(f, "database error: {error}"),
        }
    }
}

impl std::error::Error for TokenError {}

impl From<sqlx::Error> for TokenError {
    fn from(error: sqlx::Error) -> Self {
        TokenError::Db(error)
    }
}

/// Whether the character holds a token with the scope
/// (`hasEsiTokenWithScope`).
pub async fn has_scope(pool: &PgPool, character_id: i64, scope: &str) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        "select exists (select 1 from esi_tokens where character_id = $1 and $2 = any(scopes))",
    )
    .bind(character_id)
    .bind(scope)
    .fetch_one(pool)
    .await
}

/// A valid access token of the character for the scope, refreshed on the
/// way if needed (or if `force`). `Ok(None)` when the character holds no
/// such token.
pub async fn access_token(
    pool: &PgPool,
    sso: &SsoClient,
    character_id: i64,
    scope: &str,
    now: OffsetDateTime,
    force: bool,
) -> Result<Option<AccessToken>, TokenError> {
    // The newest token carrying the scope (`latest()->first()`).
    let row: Option<(i64, String, String, OffsetDateTime)> = sqlx::query_as(
        "select id, access_token, refresh_token, expires_at from esi_tokens
         where character_id = $1 and $2 = any(scopes)
         order by created_at desc, id desc
         limit 1",
    )
    .bind(character_id)
    .bind(scope)
    .fetch_optional(pool)
    .await?;
    let Some((token_id, access, refresh, expires_at)) = row else {
        return Ok(None);
    };
    if !force && expires_at - EXPIRY_BUFFER >= now {
        return Ok(Some(AccessToken {
            token_id,
            access_token: access,
            refreshed: false,
        }));
    }
    match sso.refresh(&refresh).await {
        Ok(tokens) => {
            sqlx::query(
                "update esi_tokens set access_token = $2, refresh_token = $3, expires_at = $4,
                    updated_at = $5
                 where id = $1",
            )
            .bind(token_id)
            .bind(&tokens.access_token)
            .bind(&tokens.refresh_token)
            .bind(now + Duration::seconds(tokens.expires_in.unwrap_or(0)))
            .bind(now)
            .execute(pool)
            .await?;
            Ok(Some(AccessToken {
                token_id,
                access_token: tokens.access_token,
                refreshed: true,
            }))
        }
        Err(RefreshError::Dead(reason)) => {
            tracing::warn!(character_id, token_id, %reason, "dead token deleted");
            delete(pool, token_id).await?;
            Err(TokenError::Dead(reason))
        }
        Err(RefreshError::Transient(reason)) => {
            tracing::warn!(character_id, token_id, %reason, "token refresh failed, kept");
            mark_failing(pool, token_id, &reason, now).await?;
            Err(TokenError::Transient(reason))
        }
    }
}

pub async fn delete(pool: &PgPool, token_id: i64) -> sqlx::Result<()> {
    sqlx::query("delete from esi_tokens where id = $1")
        .bind(token_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Marks a token failing; `failing_since` keeps the first failure in a
/// row.
pub async fn mark_failing(
    pool: &PgPool,
    token_id: i64,
    reason: &str,
    now: OffsetDateTime,
) -> sqlx::Result<()> {
    sqlx::query(
        "update esi_tokens set failing_since = coalesce(failing_since, $2), last_failure = $3
         where id = $1",
    )
    .bind(token_id)
    .bind(now)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

/// Clears a token's failure mark after it worked.
pub async fn mark_working(pool: &PgPool, token_id: i64) -> sqlx::Result<()> {
    sqlx::query(
        "update esi_tokens set failing_since = null, last_failure = null
         where id = $1 and (failing_since is not null or last_failure is not null)",
    )
    .bind(token_id)
    .execute(pool)
    .await?;
    Ok(())
}

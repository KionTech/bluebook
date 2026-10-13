//! The stored EVE SSO tokens, as `nicolaskion/eve`'s connector used them:
//! the character's newest token carrying the scope, refreshed first when
//! it expires within five minutes (the rotated refresh token stored), and
//! deleted when EVE rejects the refresh. ESI's 401 and 403 delete the token
//! too (`esi.rs`).

use std::fmt;

use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

use super::sso::{RefreshError, SsoClient};

/// A token counts as expired this long before its `expires_at`.
const EXPIRY_BUFFER: Duration = Duration::minutes(5);

/// A usable access token and the row behind it.
#[derive(Debug, Clone)]
pub struct AccessToken {
    pub token_id: i64,
    pub access_token: String,
}

#[derive(Debug)]
pub enum TokenError {
    /// EVE rejected the refresh; the stored token has been deleted.
    Rejected(u16),
    /// EVE could not be reached; the token was left alone.
    Network(String),
    Db(sqlx::Error),
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenError::Rejected(status) => write!(f, "token refresh rejected ({status})"),
            TokenError::Network(error) => write!(f, "SSO unreachable: {error}"),
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
/// way if needed. `Ok(None)` when the character holds no such token.
pub async fn access_token(
    pool: &PgPool,
    sso: &SsoClient,
    character_id: i64,
    scope: &str,
    now: OffsetDateTime,
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
    if expires_at - EXPIRY_BUFFER >= now {
        return Ok(Some(AccessToken {
            token_id,
            access_token: access,
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
            }))
        }
        Err(RefreshError::Rejected(status)) => {
            delete(pool, token_id).await?;
            Err(TokenError::Rejected(status))
        }
        Err(RefreshError::Network(error)) => Err(TokenError::Network(error)),
    }
}

pub async fn delete(pool: &PgPool, token_id: i64) -> sqlx::Result<()> {
    sqlx::query("delete from esi_tokens where id = $1")
        .bind(token_id)
        .execute(pool)
        .await?;
    Ok(())
}

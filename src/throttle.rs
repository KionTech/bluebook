//! Keys that hold something back until they expire: the hourly "source unreadable" alert and the mail to a
//! lapsed character every three days.

use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

/// Takes the key for `ttl` unless it is held; true when this call took it.
pub async fn add(
    pool: &PgPool,
    key: &str,
    now: OffsetDateTime,
    ttl: Duration,
) -> sqlx::Result<bool> {
    let taken = sqlx::query(
        "insert into throttles (key, expires_at) values ($1, $2)
         on conflict (key) do update set expires_at = excluded.expires_at
         where throttles.expires_at <= $3",
    )
    .bind(key)
    .bind(now + ttl)
    .bind(now)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(taken == 1)
}

/// Drops the expired keys; returns how many.
pub async fn delete_expired(pool: &PgPool, now: OffsetDateTime) -> sqlx::Result<u64> {
    Ok(sqlx::query("delete from throttles where expires_at <= $1")
        .bind(now)
        .execute(pool)
        .await?
        .rows_affected())
}

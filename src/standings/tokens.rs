//! `standings:check-tokens`, daily: every character that syncs but lost
//! its write token gets an EVE mail from the source's reader, at most
//! once every three days. Only characters on an account (FIXES.md,
//! "Removed characters were mailed about lapsed tokens forever").

use time::Duration;

use crate::auth::tokens::has_scope;
use crate::jobs::{self, Job};
use crate::scopes::{SEND_MAIL, WRITE_CHARACTER_CONTACTS};
use crate::server::AppState;
use crate::standings::{Source, source};
use crate::throttle;

/// How long a mailed character is left alone.
const MAIL_EVERY: Duration = Duration::days(3);

/// Runs the command; returns its exit code (always 0).
pub async fn check(state: &AppState) -> i32 {
    if let Err(error) = try_check(state).await {
        tracing::error!(%error, "token check failed");
    }
    0
}

async fn try_check(state: &AppState) -> sqlx::Result<()> {
    let Some(source) = Source::current(&state.pool).await? else {
        tracing::info!("No standings source is configured.");
        return Ok(());
    };
    let sender = match source::reader(state, &source).await? {
        Some(sender) if has_scope(&state.pool, sender, SEND_MAIL).await? => sender,
        _ => {
            tracing::warn!("The source character cannot send mail; skipping token check.");
            return Ok(());
        }
    };
    let lapsed: Vec<(i64, Option<i64>, Option<i64>)> = sqlx::query_as(
        "select c.id, c.corporation_id, c.alliance_id from characters c
         where c.should_sync and c.user_id is not null
            and not exists (select 1 from esi_tokens t where t.character_id = c.id and $1 = any(t.scopes))
         order by c.id",
    )
    .bind(WRITE_CHARACTER_CONTACTS)
    .fetch_all(&state.pool)
    .await?;
    let mut queued = 0;
    for (character_id, corporation_id, alliance_id) in lapsed {
        // Characters that inherit the standings need no token of their own.
        if source.covers(corporation_id, alliance_id) {
            continue;
        }
        let key = format!("standings:token-mail:{character_id}");
        if !throttle::add(&state.pool, &key, state.clock.now(), MAIL_EVERY).await? {
            continue;
        }
        jobs::enqueue(
            state,
            &Job::TokenMail {
                sender_id: sender,
                recipient_id: character_id,
            },
        )
        .await?;
        queued += 1;
    }
    tracing::info!("Queued {queued} token-expiry mail(s).");
    Ok(())
}

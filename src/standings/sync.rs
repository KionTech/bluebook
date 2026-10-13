//! `standings:sync`, every five minutes (and the admin's "Sync now", with
//! `force`): refresh the source and, when it changed, queue a sync for
//! every character that syncs and does not inherit the source in game.
//! Skipped during EVE's downtime. An unreadable source alerts Discord at
//! most once per clock hour.

use time::Duration;

use crate::jobs::{self, Job};
use crate::server::AppState;
use crate::standings::{Source, source};
use crate::throttle;

/// Runs the command; returns its exit code (1: no source, or it could not
/// be read).
pub async fn run(state: &AppState, force: bool) -> i32 {
    match try_run(state, force).await {
        Ok(code) => code,
        Err(error) => {
            tracing::error!(%error, "standings sync failed");
            1
        }
    }
}

async fn try_run(state: &AppState, force: bool) -> sqlx::Result<i32> {
    let now = state.clock.now();
    if crate::downtime::is_active(now) {
        tracing::info!("Skipping standings sync during EVE downtime.");
        return Ok(0);
    }
    let Some(changed) = source::refresh(state).await else {
        let key = format!(
            "standings:source-unreadable:{:04}{:02}{:02}{:02}",
            now.year(),
            u8::from(now.month()),
            now.day(),
            now.hour()
        );
        if Source::current(&state.pool).await?.is_some()
            && throttle::add(&state.pool, &key, now, Duration::hours(1)).await?
        {
            jobs::enqueue(state, &Job::DiscordUnreadable).await?;
        }
        tracing::warn!("No standings source is configured or the source could not be read.");
        return Ok(1);
    };
    if !changed && !force {
        tracing::info!("Source standings are unchanged; no character syncs queued.");
        return Ok(0);
    }
    let queued = queue_character_syncs(state, None).await?;
    tracing::info!("Queued {queued} character sync(s).");
    Ok(0)
}

/// Queues a sync for every character that syncs (of one account, or all)
/// and does not inherit the source; returns how many.
pub async fn queue_character_syncs(state: &AppState, user_id: Option<i64>) -> sqlx::Result<usize> {
    let source = Source::current(&state.pool).await?;
    let mut queued = 0;
    for (character_id, corporation_id, alliance_id) in
        crate::standings::syncable(&state.pool, user_id).await?
    {
        if source
            .as_ref()
            .is_some_and(|source| source.covers(corporation_id, alliance_id))
        {
            continue;
        }
        jobs::enqueue(state, &Job::CharacterSync { character_id }).await?;
        queued += 1;
    }
    Ok(queued)
}

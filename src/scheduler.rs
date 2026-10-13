//! The schedule of routes/console.php, in UTC: `standings:sync` every five
//! minutes, `characters:sync-affiliations` hourly at :37 and
//! `standings:check-tokens` daily at 00:00, on wall clock boundaries (the
//! dashboard's countdown is the last sync plus five minutes). Only with
//! SCHEDULER_ENABLED (FIXES.md, "The schedule is off unless switched on").
//! A task still running when its next run is due skips that run
//! (`withoutOverlapping`). The tasks also run once as commands (src/bin).

use std::sync::Arc;
use std::time::Duration as StdDuration;

use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;

use crate::server::AppState;

/// A scheduled task: its name and its cron expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Task {
    pub name: &'static str,
    pub cron: &'static str,
}

pub fn tasks() -> Vec<Task> {
    vec![
        Task {
            name: "standings:sync",
            cron: "*/5 * * * *",
        },
        Task {
            name: "characters:sync-affiliations",
            cron: "37 * * * *",
        },
        Task {
            name: "standings:check-tokens",
            cron: "0 0 * * *",
        },
    ]
}

/// Whether a cron field (`*`, `*/n` or a number) matches a value.
fn field_matches(field: &str, value: u8) -> bool {
    if field == "*" {
        return true;
    }
    if let Some(step) = field.strip_prefix("*/") {
        return step
            .parse::<u8>()
            .is_ok_and(|step| step > 0 && value.is_multiple_of(step));
    }
    field.parse::<u8>().is_ok_and(|wanted| wanted == value)
}

/// The first minute after `after` the task's minute and hour fields
/// match (the schedule uses no day, month or weekday fields).
pub fn next_run(task: &Task, after: OffsetDateTime) -> OffsetDateTime {
    let fields: Vec<&str> = task.cron.split_whitespace().collect();
    let (minute, hour) = (fields[0], fields[1]);
    let start = after
        .replace_second(0)
        .expect("a second")
        .replace_nanosecond(0)
        .expect("a nanosecond");
    let mut candidate = start + Duration::minutes(1);
    // A day and a minute cover every minute and hour pattern.
    for _ in 0..=(24 * 60 + 1) {
        if field_matches(minute, candidate.minute()) && field_matches(hour, candidate.hour()) {
            return candidate;
        }
        candidate += Duration::minutes(1);
    }
    panic!("the cron expression {} never matches", task.cron);
}

async fn run(state: &AppState, task: &Task) {
    let code = match task.name {
        "standings:sync" => crate::standings::sync::run(state, false).await,
        "characters:sync-affiliations" => crate::entities::sync_affiliations(state).await,
        _ => crate::standings::tokens::check(state).await,
    };
    if code != 0 {
        tracing::warn!(task = task.name, code, "scheduled command failed");
    }
}

fn until(state: &AppState, moment: OffsetDateTime) -> StdDuration {
    StdDuration::from_millis((moment - state.clock.now()).whole_milliseconds().max(0) as u64)
}

/// Starts a loop per task when the schedule is switched on.
pub fn spawn(state: &AppState) -> Vec<tokio::task::JoinHandle<()>> {
    if !state.config.scheduler_enabled {
        tracing::info!("the schedule is off (SCHEDULER_ENABLED)");
        return Vec::new();
    }
    tasks()
        .into_iter()
        .map(|task| {
            let state = state.clone();
            let running = Arc::new(Mutex::new(()));
            tokio::spawn(async move {
                loop {
                    let at = next_run(&task, state.clock.now());
                    tokio::time::sleep(until(&state, at)).await;
                    let Ok(guard) = running.clone().try_lock_owned() else {
                        tracing::info!(task = task.name, "still running, skipped");
                        continue;
                    };
                    let state = state.clone();
                    tokio::spawn(async move {
                        run(&state, &task).await;
                        drop(guard);
                    });
                }
            })
        })
        .collect()
}

/// How long a failed job is kept for a look.
const FAILED_JOBS_KEPT_FOR: Duration = Duration::days(7);

/// Hourly, always: expired sessions and throttle keys, and failed jobs
/// older than a week, go (Laravel's lottery and nobody did this).
pub fn spawn_housekeeping(state: &AppState) -> tokio::task::JoinHandle<()> {
    let state = state.clone();
    tokio::spawn(async move {
        let mut hourly = tokio::time::interval(StdDuration::from_secs(3600));
        loop {
            hourly.tick().await;
            let now = state.clock.now();
            if let Err(error) = crate::auth::session::delete_expired(
                &state.pool,
                now,
                state.config.session_lifetime_days,
            )
            .await
            {
                tracing::error!(%error, "expired sessions not deleted");
            }
            if let Err(error) = crate::throttle::delete_expired(&state.pool, now).await {
                tracing::error!(%error, "expired throttles not deleted");
            }
            if let Err(error) = sqlx::query("delete from jobs where failed_at < $1")
                .bind(now - FAILED_JOBS_KEPT_FOR)
                .execute(&state.pool)
                .await
            {
                tracing::error!(%error, "old failed jobs not deleted");
            }
        }
    })
}

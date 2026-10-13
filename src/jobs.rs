//! The job queue in Postgres: character syncs, the lapsed token mails and
//! the Discord messages. Workers in the api claim due jobs with `for
//! update skip locked`. A job gets one try; done jobs
//! are deleted, failed ones kept with their error (and dropped after a
//! week by the housekeeping).

use std::time::Duration as StdDuration;

use serde_json::{Value, json};
use sqlx::PgPool;

use crate::server::AppState;

/// How long an idle worker waits before it looks again (`--sleep=3`).
const IDLE_WAIT: StdDuration = StdDuration::from_secs(3);

/// A queued job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Job {
    CharacterSync { character_id: i64 },
    DiscordRequest { standing_request_id: i64 },
    DiscordUnreadable,
    TokenMail { sender_id: i64, recipient_id: i64 },
}

impl Job {
    fn kind(&self) -> &'static str {
        match self {
            Job::CharacterSync { .. } => "character_sync",
            Job::DiscordRequest { .. } => "discord_request",
            Job::DiscordUnreadable => "discord_unreadable",
            Job::TokenMail { .. } => "token_mail",
        }
    }

    fn payload(&self) -> Value {
        match self {
            Job::CharacterSync { character_id } => json!({ "character_id": character_id }),
            Job::DiscordRequest {
                standing_request_id,
            } => {
                json!({ "standing_request_id": standing_request_id })
            }
            Job::DiscordUnreadable => json!({}),
            Job::TokenMail {
                sender_id,
                recipient_id,
            } => json!({ "sender_id": sender_id, "recipient_id": recipient_id }),
        }
    }

    fn from_row(kind: &str, payload: &Value) -> Option<Job> {
        let id = |key: &str| payload.get(key).and_then(Value::as_i64);
        Some(match kind {
            "character_sync" => Job::CharacterSync {
                character_id: id("character_id")?,
            },
            "discord_request" => Job::DiscordRequest {
                standing_request_id: id("standing_request_id")?,
            },
            "discord_unreadable" => Job::DiscordUnreadable,
            "token_mail" => Job::TokenMail {
                sender_id: id("sender_id")?,
                recipient_id: id("recipient_id")?,
            },
            _ => return None,
        })
    }
}

pub async fn enqueue(state: &AppState, job: &Job) -> sqlx::Result<()> {
    let now = state.clock.now();
    sqlx::query(
        "insert into jobs (kind, payload, available_at, created_at) values ($1, $2, $3, $3)",
    )
    .bind(job.kind())
    .bind(job.payload())
    .bind(now)
    .execute(&state.pool)
    .await?;
    state.jobs.notify_one();
    Ok(())
}

#[derive(sqlx::FromRow)]
struct Claimed {
    id: i64,
    kind: String,
    payload: sqlx::types::Json<Value>,
}

/// Takes the next due job, if any.
async fn claim(pool: &PgPool, now: time::OffsetDateTime) -> sqlx::Result<Option<Claimed>> {
    sqlx::query_as(
        "update jobs set reserved_at = $1, attempts = attempts + 1
         where id = (
             select id from jobs
             where failed_at is null and reserved_at is null and available_at <= $1
             order by available_at, id
             for update skip locked
             limit 1
         )
         returning id, kind, payload",
    )
    .bind(now)
    .fetch_optional(pool)
    .await
}

async fn handle(state: &AppState, job: &Job) -> Result<(), String> {
    match job {
        Job::CharacterSync { character_id } => {
            crate::standings::character::sync(state, *character_id)
                .await
                .map_err(|error| match error {
                    sqlx::Error::RowNotFound => format!("character {character_id} does not exist"),
                    other => other.to_string(),
                })
        }
        Job::DiscordRequest {
            standing_request_id,
        } => crate::discord::notify_request(state, *standing_request_id).await,
        Job::DiscordUnreadable => crate::discord::notify_unreadable(state).await,
        Job::TokenMail {
            sender_id,
            recipient_id,
        } => token_mail(state, *sender_id, *recipient_id).await,
    }
}

/// `SendTokenExpiredMail`: the mail from the source's reader to a
/// character whose write token lapsed, with the sync grant link.
async fn token_mail(state: &AppState, sender_id: i64, recipient_id: i64) -> Result<(), String> {
    let names: Vec<(i64, Option<String>)> =
        sqlx::query_as("select id, name from characters where id = any($1)")
            .bind(vec![sender_id, recipient_id])
            .fetch_all(&state.pool)
            .await
            .map_err(|error| error.to_string())?;
    let sender_exists = names.iter().any(|(id, _)| *id == sender_id);
    let Some(recipient) = names.iter().find(|(id, _)| *id == recipient_id) else {
        return Ok(());
    };
    if !sender_exists
        || !crate::auth::tokens::has_scope(&state.pool, sender_id, crate::scopes::SEND_MAIL)
            .await
            .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let link = crate::scopes::grant_url(&state.config.app_url, &crate::scopes::SYNC_SCOPES);
    let body = format!(
        "Hello {},\n\nYour contact permissions for Bluebook have lapsed, so your standings can't be synced. Please grant them again here:\n{link}\n\nso the service won't get interrupted.",
        recipient.1.as_deref().unwrap_or_default()
    );
    let subject = "Bluebook \u{2014} re-authentication required";
    if let Err(error) = crate::esi::send_mail(state, sender_id, recipient_id, subject, &body).await
    {
        tracing::warn!(%error, recipient_id, "token mail not sent");
    }
    Ok(())
}

/// Runs one claimed job to its end: deleted when done, kept with its error
/// when it failed.
async fn process(state: &AppState, claimed: Claimed) {
    let outcome = match Job::from_row(&claimed.kind, &claimed.payload.0) {
        Some(job) => handle(state, &job).await,
        None => Err(format!("unknown job {}", claimed.kind)),
    };
    let stored = match &outcome {
        Ok(()) => {
            sqlx::query("delete from jobs where id = $1")
                .bind(claimed.id)
                .execute(&state.pool)
                .await
        }
        Err(error) => {
            tracing::error!(job = claimed.id, kind = %claimed.kind, %error, "job failed");
            sqlx::query(
                "update jobs set reserved_at = null, failed_at = $2, last_error = $3 where id = $1",
            )
            .bind(claimed.id)
            .bind(state.clock.now())
            .bind(error)
            .execute(&state.pool)
            .await
        }
    };
    if let Err(error) = stored {
        tracing::error!(job = claimed.id, %error, "job outcome not stored");
    }
}

/// Runs due jobs until none is left; returns how many ran.
pub async fn run_pending(state: &AppState) -> usize {
    let mut ran = 0;
    loop {
        match claim(&state.pool, state.clock.now()).await {
            Ok(Some(claimed)) => {
                process(state, claimed).await;
                ran += 1;
            }
            Ok(None) => return ran,
            Err(error) => {
                tracing::error!(%error, "job not claimed");
                return ran;
            }
        }
    }
}

/// Jobs a stopped api had taken are free again (one api process runs the
/// workers).
pub async fn release_reserved(pool: &PgPool) -> sqlx::Result<u64> {
    Ok(sqlx::query(
        "update jobs set reserved_at = null where reserved_at is not null and failed_at is null",
    )
    .execute(pool)
    .await?
    .rows_affected())
}

/// Starts the workers: each runs due jobs, then waits for a new one or
/// three seconds.
pub fn spawn_workers(state: &AppState) -> Vec<tokio::task::JoinHandle<()>> {
    (0..state.config.queue_workers)
        .map(|_| {
            let state = state.clone();
            tokio::spawn(async move {
                loop {
                    run_pending(&state).await;
                    let _ = tokio::time::timeout(IDLE_WAIT, state.jobs.notified()).await;
                }
            })
        })
        .collect()
}

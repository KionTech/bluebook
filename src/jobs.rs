//! The job queue. Not ported yet.

use crate::server::AppState;

/// A queued job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Job {
    CharacterSync { character_id: i64 },
    DiscordRequest { standing_request_id: i64 },
    DiscordUnreadable,
    TokenMail { sender_id: i64, recipient_id: i64 },
}

pub async fn enqueue(_state: &AppState, _job: &Job) -> sqlx::Result<()> {
    todo!("the job queue")
}

pub async fn run_pending(_state: &AppState) -> usize {
    todo!("the job queue")
}

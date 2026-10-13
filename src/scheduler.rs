//! The schedule. Not ported yet.

use time::OffsetDateTime;

/// A scheduled task: its name and its cron expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Task {
    pub name: &'static str,
    pub cron: &'static str,
}

pub fn tasks() -> Vec<Task> {
    todo!("the schedule")
}

pub fn next_run(_task: &Task, _after: OffsetDateTime) -> OffsetDateTime {
    todo!("the schedule")
}

/// Starts the schedule's loops when it is switched on.
pub fn spawn(_state: &crate::server::AppState) -> Vec<tokio::task::JoinHandle<()>> {
    todo!("the schedule")
}

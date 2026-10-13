//! tests/Feature/ScheduleTest.php: the three scheduled tasks and their
//! cadence; and their alignment on wall clock boundaries, EVE's downtime
//! window and the switch that turns the schedule on.

use time::macros::datetime;

use bluebook::scheduler::{Task, next_run, tasks};

fn task(name: &str) -> Task {
    tasks()
        .into_iter()
        .find(|task| task.name == name)
        .unwrap_or_else(|| panic!("no task {name}"))
}

#[test]
fn schedules_the_standings_sync_every_five_minutes() {
    assert_eq!(task("standings:sync").cron, "*/5 * * * *");
}

#[test]
fn schedules_the_affiliation_sync_hourly() {
    assert_eq!(task("characters:sync-affiliations").cron, "37 * * * *");
}

#[test]
fn schedules_the_token_check_daily() {
    assert_eq!(task("standings:check-tokens").cron, "0 0 * * *");
}

#[test]
fn the_tasks_run_on_wall_clock_boundaries() {
    let sync = task("standings:sync");
    assert_eq!(
        next_run(&sync, datetime!(2026-06-25 14:00:00 UTC)),
        datetime!(2026-06-25 14:05:00 UTC)
    );
    assert_eq!(
        next_run(&sync, datetime!(2026-06-25 14:03:59 UTC)),
        datetime!(2026-06-25 14:05:00 UTC)
    );
    assert_eq!(
        next_run(&sync, datetime!(2026-06-25 23:58:00 UTC)),
        datetime!(2026-06-26 00:00:00 UTC)
    );

    let affiliations = task("characters:sync-affiliations");
    assert_eq!(
        next_run(&affiliations, datetime!(2026-06-25 14:36:59 UTC)),
        datetime!(2026-06-25 14:37:00 UTC)
    );
    assert_eq!(
        next_run(&affiliations, datetime!(2026-06-25 14:37:00 UTC)),
        datetime!(2026-06-25 15:37:00 UTC)
    );

    let tokens = task("standings:check-tokens");
    assert_eq!(
        next_run(&tokens, datetime!(2026-06-25 14:00:00 UTC)),
        datetime!(2026-06-26 00:00:00 UTC)
    );
    assert_eq!(
        next_run(&tokens, datetime!(2026-12-31 23:59:59 UTC)),
        datetime!(2027-01-01 00:00:00 UTC)
    );
}

#[test]
fn the_downtime_window_includes_its_start_and_excludes_its_end() {
    use bluebook::downtime::is_active;
    assert!(!is_active(datetime!(2026-06-25 10:59:59 UTC)));
    assert!(is_active(datetime!(2026-06-25 11:00:00 UTC)));
    assert!(is_active(datetime!(2026-06-25 11:14:59 UTC)));
    assert!(!is_active(datetime!(2026-06-25 11:15:00 UTC)));
}

/// FIXES.md, "The schedule is off unless switched on".
#[tokio::test]
async fn the_schedule_is_off_unless_enabled() {
    let app = crate::common::TestApp::new().await;
    assert!(!app.config.scheduler_enabled);
    // Spawning the schedule without the switch starts no task.
    assert_eq!(bluebook::scheduler::spawn(&app.state).len(), 0);
}

//! The job queue: a job runs once, is deleted when done and kept with its
//! error when it fails, and two workers never take the same job.

use serde_json::json;

use bluebook::jobs::Job;

use crate::common::factories::NewRequest;
use crate::common::{TestApp, assert_json_eq};

#[tokio::test]
async fn a_job_runs_once_and_is_deleted() {
    let app = TestApp::new().await;
    app.discord_settings(Some(&app.discord.webhook()), None)
        .await;
    let request = app.standing_request(NewRequest::default()).await;
    bluebook::jobs::enqueue(
        &app.state,
        &Job::DiscordRequest {
            standing_request_id: request,
        },
    )
    .await
    .expect("a queued job");

    assert_eq!(bluebook::jobs::run_pending(&app.state).await, 1);
    assert_eq!(bluebook::jobs::run_pending(&app.state).await, 0);

    assert_eq!(app.discord.requests().len(), 1);
    assert_eq!(app.count("jobs").await, 0);
}

#[tokio::test]
async fn a_failing_job_is_kept_with_its_error_and_not_tried_again() {
    let app = TestApp::new().await;
    bluebook::jobs::enqueue(
        &app.state,
        &Job::DiscordRequest {
            standing_request_id: 404,
        },
    )
    .await
    .expect("a queued job");

    bluebook::jobs::run_pending(&app.state).await;
    bluebook::jobs::run_pending(&app.state).await;

    assert_json_eq(
        &json!([{
            "kind": "discord_request",
            "attempts": 1,
            "failed": true,
            "last_error": "standing request 404 does not exist",
        }]),
        &json!(
            app.rows(
                "select kind, attempts, failed_at is not null as failed, last_error from jobs"
            )
            .await
        ),
    );
}

#[tokio::test]
async fn two_workers_never_take_the_same_job() {
    let app = TestApp::new().await;
    app.discord_settings(Some(&app.discord.webhook()), None)
        .await;
    for _ in 0..12 {
        let request = app.standing_request(NewRequest::default()).await;
        bluebook::jobs::enqueue(
            &app.state,
            &Job::DiscordRequest {
                standing_request_id: request,
            },
        )
        .await
        .expect("a queued job");
    }

    let (first, second) = tokio::join!(
        bluebook::jobs::run_pending(&app.state),
        bluebook::jobs::run_pending(&app.state)
    );

    assert_eq!(first + second, 12);
    assert_eq!(app.discord.requests().len(), 12);
}

#[tokio::test]
async fn the_payloads_name_their_rows() {
    let app = TestApp::new().await;
    for job in [
        Job::CharacterSync { character_id: 1 },
        Job::DiscordRequest {
            standing_request_id: 2,
        },
        Job::DiscordUnreadable,
        Job::TokenMail {
            sender_id: 3,
            recipient_id: 4,
        },
    ] {
        bluebook::jobs::enqueue(&app.state, &job)
            .await
            .expect("a queued job");
    }

    assert_json_eq(
        &json!([
            ["character_sync", { "character_id": 1 }],
            ["discord_request", { "standing_request_id": 2 }],
            ["discord_unreadable", {}],
            ["token_mail", { "sender_id": 3, "recipient_id": 4 }],
        ]),
        &json!(app.jobs().await),
    );
}

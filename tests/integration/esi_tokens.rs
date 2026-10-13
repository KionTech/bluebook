//! The stored EVE SSO tokens as the ESI calls use them: the newest token
//! with the scope, the refresh of one expiring within five minutes, and
//! a failed refresh.

use time::Duration;

use crate::common::factories::NewCharacter;
use crate::common::{READ_CORPORATION_CONTACTS, TestApp, assert_json_eq};

async fn readable_source(app: &mut TestApp, expires_in: Duration) -> i64 {
    app.corporation(2000, Some("Source Corp"), None, None).await;
    let admin = app
        .character(NewCharacter {
            corporation_id: Some(2000),
            ..NewCharacter::default()
        })
        .await;
    app.token(
        admin,
        &[READ_CORPORATION_CONTACTS],
        app.clock.now() + expires_in,
    )
    .await;
    app.admins(&[admin]);
    app.source("corporation", 2000).await;
    app.esi.contacts("corporations/2000", &[]);
    admin
}

#[tokio::test]
async fn a_token_valid_for_five_more_minutes_is_used_as_it_is() {
    let mut app = TestApp::new().await;
    readable_source(&mut app, Duration::minutes(5)).await;

    bluebook::standings::source::refresh(&app.state).await;

    assert_eq!(app.sso.requests().len(), 0);
}

#[tokio::test]
async fn a_token_expiring_within_five_minutes_is_refreshed_first() {
    let mut app = TestApp::new().await;
    readable_source(&mut app, Duration::minutes(5) - Duration::seconds(1)).await;

    bluebook::standings::source::refresh(&app.state).await;

    let refresh = &app.sso.requests()[0];
    assert_eq!(refresh.path, "/v2/oauth/token");
    assert_eq!(
        refresh.header("authorization"),
        Some("Basic dGVzdC1jbGllbnQtaWQ6dGVzdC1jbGllbnQtc2VjcmV0")
    );
    assert!(
        refresh
            .text()
            .starts_with("grant_type=refresh_token&refresh_token=refresh-token-")
    );
    assert_eq!(
        app.esi.requests()[0].header("authorization"),
        Some("Bearer refreshed-access-1")
    );
    assert_json_eq(
        &serde_json::json!([{
            "access_token": "refreshed-access-1",
            "refresh_token": "refreshed-refresh-1",
            "expires_at": "2026-06-25T14:19:59+00:00",
        }]),
        &serde_json::json!(app.rows(
            "select access_token, refresh_token,
                to_char(expires_at at time zone 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"+00:00\"') as expires_at
             from esi_tokens"
        ).await),
    );
}

#[tokio::test]
async fn the_refresh_is_tried_five_times_on_a_server_error() {
    let mut app = TestApp::new().await;
    readable_source(&mut app, Duration::minutes(-1)).await;
    app.sso.queue_refresh(&[502, 502, 502, 502]);

    assert_eq!(
        bluebook::standings::source::refresh(&app.state).await,
        Some(false)
    );

    assert_eq!(app.sso.requests().len(), 5);
    assert_eq!(app.count("esi_tokens").await, 1);
}

/// EVE answers `invalid_grant` for a refresh token that was revoked or
/// expired: the only refresh answer that deletes the token.
#[tokio::test]
async fn a_refresh_answered_with_invalid_grant_deletes_the_token() {
    let mut app = TestApp::new().await;
    readable_source(&mut app, Duration::minutes(-1)).await;
    app.sso.queue_refresh(&[400]);

    assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);

    assert_eq!(app.sso.requests().len(), 1);
    assert_eq!(app.count("esi_tokens").await, 0);
}

/// FIXES.md, "Tokens were thrown away on any failure".
#[tokio::test]
async fn a_refresh_failing_five_times_keeps_the_token() {
    let mut app = TestApp::new().await;
    readable_source(&mut app, Duration::minutes(-1)).await;
    app.sso.queue_refresh(&[503, 503, 503, 503, 503]);

    assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);

    assert_eq!(app.sso.requests().len(), 5);
    assert_json_eq(
        &serde_json::json!([{
            "failing_since": "2026-06-25T14:00:00+00:00",
            "last_failure": "token refresh failed (503)",
        }]),
        &serde_json::json!(app.rows(
            "select to_char(failing_since at time zone 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"+00:00\"') as failing_since,
                last_failure
             from esi_tokens"
        ).await),
    );
}

#[tokio::test]
async fn other_refresh_errors_keep_the_token() {
    for (status, body) in [
        (400, serde_json::json!({ "error": "invalid_request" })),
        (401, serde_json::json!({ "error": "invalid_client" })),
        (420, serde_json::json!({ "error": "Error limited" })),
        (429, serde_json::json!({})),
    ] {
        let mut app = TestApp::new().await;
        readable_source(&mut app, Duration::minutes(-1)).await;
        app.sso.queue_refresh_with(status, body);

        assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);

        assert_eq!(app.sso.requests().len(), 1, "{status}");
        assert_eq!(app.count("esi_tokens").await, 1, "{status}");
        assert_eq!(app.esi.requests().len(), 0, "{status}");
    }
}

#[tokio::test]
async fn an_unreachable_sso_keeps_the_token() {
    let mut app = TestApp::new().await;
    readable_source(&mut app, Duration::minutes(-1)).await;
    app.reconfigure(|config| config.eve.sso_base_url = "http://127.0.0.1:1".to_owned());

    assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);

    assert_eq!(app.count("esi_tokens").await, 1);
    assert_eq!(
        app.scalar("select failing_since is not null from esi_tokens")
            .await,
        serde_json::json!(true)
    );
}

#[tokio::test]
async fn a_refresh_that_works_again_clears_the_failure_mark() {
    let mut app = TestApp::new().await;
    readable_source(&mut app, Duration::minutes(-1)).await;
    app.sso.queue_refresh(&[503, 503, 503, 503, 503]);
    bluebook::standings::source::refresh(&app.state).await;

    assert_eq!(
        bluebook::standings::source::refresh(&app.state).await,
        Some(false)
    );

    assert_eq!(
        app.scalar("select failing_since is null and last_failure is null from esi_tokens")
            .await,
        serde_json::json!(true)
    );
}

/// FIXES.md, "A failed token refresh still sent the request with the dead
/// token".
#[tokio::test]
async fn a_failed_refresh_sends_no_request() {
    let mut app = TestApp::new().await;
    readable_source(&mut app, Duration::minutes(-1)).await;
    app.sso.queue_refresh(&[400]);

    bluebook::standings::source::refresh(&app.state).await;

    assert_eq!(app.esi.requests().len(), 0);
}

#[tokio::test]
async fn the_newest_token_with_the_scope_is_used() {
    let mut app = TestApp::new().await;
    let admin = readable_source(&mut app, Duration::hours(1)).await;
    app.clock.advance(Duration::seconds(1));
    let newer = app
        .token(
            admin,
            &[READ_CORPORATION_CONTACTS],
            app.clock.now() + Duration::hours(1),
        )
        .await;

    bluebook::standings::source::refresh(&app.state).await;

    let token = app
        .scalar(&format!(
            "select access_token from esi_tokens where id = {newer}"
        ))
        .await;
    assert_eq!(
        app.esi.requests()[0]
            .header("authorization")
            .map(str::to_owned),
        Some(format!("Bearer {}", token.as_str().expect("a token")))
    );
}

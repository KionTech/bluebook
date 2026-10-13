//! ESI as `nicolaskion/eve` called it: the headers, the retries on a
//! server error (reads only), and the token a 401 or 403 deletes.

use serde_json::json;

use crate::common::factories::NewCharacter;
use crate::common::{
    Canned, READ_CHARACTER_CONTACTS, READ_CORPORATION_CONTACTS, TestApp, WRITE_CHARACTER_CONTACTS,
};

/// A corporation source with an admin that can read it.
async fn readable_source(app: &mut TestApp) -> i64 {
    app.corporation(2000, Some("Source Corp"), None, None).await;
    let admin = app
        .character(NewCharacter {
            corporation_id: Some(2000),
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(admin, &[READ_CORPORATION_CONTACTS]).await;
    app.admins(&[admin]);
    app.source("corporation", 2000).await;
    admin
}

fn server_errors(count: usize) -> Vec<Canned> {
    (0..count)
        .map(|_| Canned {
            status: 502,
            body: json!({ "error": "bad gateway" }),
        })
        .collect()
}

/// FIXES.md, "The ESI user agent names the site".
#[tokio::test]
async fn requests_carry_the_user_agent_and_compatibility_date() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.esi.contacts("corporations/2000", &[]);

    bluebook::standings::source::refresh(&app.state).await;

    let request = &app.esi.requests()[0];
    assert_eq!(
        request.header("user-agent"),
        Some("Bluebook (+https://bluebook.nicolaskion.dev)")
    );
    assert_eq!(request.header("x-compatibility-date"), Some("2026-07-17"));
    assert!(
        request
            .header("authorization")
            .is_some_and(|value| value.starts_with("Bearer access-token-"))
    );
}

#[tokio::test]
async fn reads_are_tried_five_times_on_a_server_error() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.esi
        .contacts("corporations/2000", &[(100, "faction", 5.0)]);
    app.esi
        .queue("GET /corporations/2000/contacts/", server_errors(4));

    assert_eq!(
        bluebook::standings::source::refresh(&app.state).await,
        Some(true)
    );
    assert_eq!(app.esi.requests().len(), 5);
}

#[tokio::test]
async fn a_read_failing_five_times_fails() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.esi
        .queue("GET /corporations/2000/contacts/", server_errors(5));
    app.esi
        .contacts("corporations/2000", &[(100, "faction", 5.0)]);

    assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);
    assert_eq!(app.esi.requests().len(), 5);
}

#[tokio::test]
async fn a_client_error_is_not_tried_again() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.esi.fail_path("/corporations/2000/contacts/", 404);

    assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);
    assert_eq!(app.esi.requests().len(), 1);
    assert_eq!(app.count("esi_tokens").await, 1);
}

#[tokio::test]
async fn contact_writes_are_never_tried_again() {
    let app = TestApp::new().await;
    let character = app
        .character(NewCharacter {
            should_sync: true,
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(character, &[WRITE_CHARACTER_CONTACTS])
        .await;
    app.source_contact(100, "faction", 5.0).await;
    app.esi
        .fail_path(&format!("POST /characters/{character}/contacts/"), 503);

    bluebook::standings::character::sync(&app.state, character)
        .await
        .expect("a sync");

    assert_eq!(app.esi.requests().len(), 1);
}

#[tokio::test]
async fn a_forbidden_answer_deletes_the_token_used() {
    let mut app = TestApp::new().await;
    let admin = readable_source(&mut app).await;
    let other = app.grant_scopes(admin, &[READ_CHARACTER_CONTACTS]).await;
    app.esi.fail_path("/corporations/2000/contacts/", 403);

    assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);

    // The corporation token went; the other one stays.
    let left: Vec<i64> = app
        .rows("select id from esi_tokens")
        .await
        .into_iter()
        .map(|row| row["id"].as_i64().expect("an id"))
        .collect();
    assert_eq!(left, vec![other]);
}

#[tokio::test]
async fn an_unauthorized_answer_deletes_the_token_used() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.esi.fail_path("/corporations/2000/contacts/", 401);

    bluebook::standings::source::refresh(&app.state).await;

    assert_eq!(app.count("esi_tokens").await, 0);
}

#[tokio::test]
async fn names_are_read_in_chunks_of_one_thousand() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    let contacts: Vec<(i64, &str, f64)> = (1..=1001)
        .map(|n| (90_200_000 + n, "character", 5.0))
        .collect();
    app.esi.contacts("corporations/2000", &contacts);

    bluebook::standings::source::refresh(&app.state).await;

    let sizes: Vec<usize> = app
        .esi
        .requests()
        .iter()
        .filter(|request| request.path == "/universe/names/")
        .map(|request| request.json().as_array().map_or(0, Vec::len))
        .collect();
    assert_eq!(sizes, vec![1000, 1]);
}

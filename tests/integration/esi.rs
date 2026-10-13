//! The ESI client: the headers, the retries on a server error (reads
//! only), and what each failure does to the token.

use serde_json::json;

use crate::common::factories::NewCharacter;
use crate::common::{
    Canned, READ_CHARACTER_CONTACTS, READ_CORPORATION_CONTACTS, TestApp, WRITE_CHARACTER_CONTACTS,
    assert_json_eq,
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
async fn a_not_found_answer_is_not_tried_again() {
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
async fn a_forbidden_answer_keeps_the_token_and_marks_the_source_unreadable() {
    let mut app = TestApp::new().await;
    let admin = readable_source(&mut app).await;
    let other = app.grant_scopes(admin, &[READ_CHARACTER_CONTACTS]).await;
    app.esi.fail_path("/corporations/2000/contacts/", 403);

    assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);

    // A missing in-game role is no dead token: both stay, unmarked.
    assert_eq!(app.count("esi_tokens").await, 2);
    assert_eq!(
        app.scalar("select count(*) from esi_tokens where failing_since is not null")
            .await,
        json!(0)
    );
    assert!(other > 0);
    assert_eq!(
        app.scalar("select unreadable_since is not null from standings_sources")
            .await,
        json!(true)
    );
    let user: i64 = sqlx::query_scalar("select user_id from characters where id = $1")
        .bind(admin)
        .fetch_one(&app.pool)
        .await
        .expect("an owner");
    let props = app
        .page(&app.sign_in(user).await, "/dashboard")
        .await
        .json();
    assert_eq!(props["auth"]["source_unreadable"], true);
}

#[tokio::test]
async fn a_read_that_works_again_clears_the_unreadable_mark() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.esi.queue(
        "GET /corporations/2000/contacts/",
        vec![Canned {
            status: 403,
            body: json!({ "error": "forbidden" }),
        }],
    );
    app.esi.contacts("corporations/2000", &[]);

    assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);
    assert_eq!(
        bluebook::standings::source::refresh(&app.state).await,
        Some(false)
    );

    assert_eq!(
        app.scalar("select unreadable_since is null from standings_sources")
            .await,
        json!(true)
    );
}

#[tokio::test]
async fn an_unauthorized_answer_refreshes_the_token_and_tries_again() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.esi
        .contacts("corporations/2000", &[(100, "faction", 5.0)]);
    app.esi.queue(
        "GET /corporations/2000/contacts/",
        vec![Canned {
            status: 401,
            body: json!({ "error": "token is expired" }),
        }],
    );

    assert_eq!(
        bluebook::standings::source::refresh(&app.state).await,
        Some(true)
    );

    assert_eq!(app.sso.requests().len(), 1);
    let requests = app.esi.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[1].header("authorization"),
        Some("Bearer refreshed-access-1")
    );
    assert_eq!(app.count("esi_tokens").await, 1);
}

#[tokio::test]
async fn an_unauthorized_answer_right_after_a_refresh_deletes_the_token() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.esi.fail_path("/corporations/2000/contacts/", 401);

    assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);

    assert_eq!(app.sso.requests().len(), 1);
    assert_eq!(app.esi.requests().len(), 2);
    assert_eq!(app.count("esi_tokens").await, 0);
}

#[tokio::test]
async fn transient_failures_keep_the_token_and_mark_it_failing() {
    for code in [420, 429, 500, 503] {
        let mut app = TestApp::new().await;
        readable_source(&mut app).await;
        app.esi.fail_path("/corporations/2000/contacts/", code);

        assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);

        assert_eq!(app.count("esi_tokens").await, 1, "{code}");
        assert_json_eq(
            &json!([{ "failing_since": "2026-06-25T14:00:00+00:00", "last_failure": format!("ESI answered {code}") }]),
            &json!(app.rows(
                "select to_char(failing_since at time zone 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"+00:00\"') as failing_since, last_failure from esi_tokens"
            ).await),
        );
    }
}

#[tokio::test]
async fn an_unreachable_esi_keeps_the_token() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.reconfigure(|config| config.esi.base_url = "http://127.0.0.1:1".to_owned());

    assert_eq!(bluebook::standings::source::refresh(&app.state).await, None);

    assert_eq!(app.count("esi_tokens").await, 1);
    assert_eq!(
        app.scalar("select failing_since is not null from esi_tokens")
            .await,
        json!(true)
    );
}

#[tokio::test]
async fn a_success_clears_the_failure_mark_but_keeps_its_start_while_failing() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.esi.contacts("corporations/2000", &[]);
    app.esi.queue(
        "GET /corporations/2000/contacts/",
        (0..5)
            .map(|_| Canned {
                status: 502,
                body: json!({}),
            })
            .collect(),
    );

    bluebook::standings::source::refresh(&app.state).await;
    let first = app.scalar("select failing_since from esi_tokens").await;
    assert!(!first.is_null());
    app.clock.advance(time::Duration::hours(1));
    app.esi.queue(
        "GET /corporations/2000/contacts/",
        (0..5)
            .map(|_| Canned {
                status: 502,
                body: json!({}),
            })
            .collect(),
    );
    bluebook::standings::source::refresh(&app.state).await;
    // Still failing: the mark keeps the first failure's time.
    assert_eq!(
        app.scalar("select failing_since from esi_tokens").await,
        first
    );

    bluebook::standings::source::refresh(&app.state).await;
    assert_eq!(
        app.scalar("select failing_since is null and last_failure is null from esi_tokens")
            .await,
        json!(true)
    );
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

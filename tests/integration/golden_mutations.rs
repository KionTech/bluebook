//! Every form the Laravel reference answered
//! (tests/fixtures/golden/mutations), as the pages send it now (FIXES.md,
//! "Forms answer JSON"): Laravel's redirect back is `{"redirect": null}`,
//! a redirect elsewhere names its path, validation errors are the 422's,
//! the flash message arrives with the next page's props. The rows, the
//! queued jobs and the ESI calls are compared as they were left.

use axum::http::Method;
use serde_json::Value;

use crate::common::golden::{self, assert_rows_eq, error_page_text};
use crate::common::{APP_URL, Browser, TestApp, assert_json_eq};

async fn browser(app: &TestApp, golden: &Value) -> Browser {
    match (golden["user"].as_i64(), golden["character"].as_i64()) {
        (Some(user), Some(character)) => app.sign_in_as(user, character).await,
        (Some(user), None) => app.sign_in(user).await,
        _ => Browser::guest(),
    }
}

/// Where a recorded redirect sends the browser now: `None` for Laravel's
/// `back()` (the page the form was on), else the site path.
fn redirect_of(location: &str, page: &str) -> Option<String> {
    if location == format!("{APP_URL}{page}") {
        return None;
    }
    let path = location.strip_prefix(APP_URL).unwrap_or(location);
    Some(if path.is_empty() {
        "/".to_owned()
    } else {
        path.to_owned()
    })
}

/// The source is set through an admin character of the account now, not
/// the active one (FIXES.md, "The source came from the active character,
/// not an admin character"): with the alt active, picking the alliance
/// does what it does with the admin character active.
fn with_fixed_source(name: &str, golden: &mut Value) {
    if name == "admin_source_no_alliance" {
        let same = golden::load("mutations", "admin_source_same");
        let active = golden["after"]["active_character"].clone();
        for key in ["after", "rows", "jobs", "http"] {
            golden[key] = same[key].clone();
        }
        golden["after"]["active_character"] = active;
    }
}

async fn check(name: &str) {
    let mut golden = golden::load("mutations", name);
    with_fixed_source(name, &mut golden);
    let app = golden::app(&golden["sql"]).await;
    golden::configure_fakes(&app, &golden["fakes"]);
    let browser = browser(&app, &golden).await;
    let method: Method = golden["method"]
        .as_str()
        .expect("a method")
        .parse()
        .expect("a method");
    let path = golden["path"].as_str().expect("a path");
    let page = golden["page"].as_str().expect("a page");
    let response = app
        .form(&browser, method, path, golden["body"].clone())
        .await;
    let status = golden["status"].as_u64().expect("a status") as u16;
    let after = &golden["after"];
    let errors = after["errors"]
        .as_object()
        .filter(|errors| !errors.is_empty());

    match status {
        302 if errors.is_some() => {
            let actual = response.assert_invalid();
            assert_json_eq(&after["errors"], &actual);
        }
        302 => {
            let location = golden["location"].as_str().expect("a location");
            if location.starts_with('/') {
                // Laravel's Route::redirect, for every method.
                response.assert_status(302);
                assert_eq!(response.location(), location);
            } else {
                let redirect = response.assert_done();
                assert_eq!(redirect, redirect_of(location, page), "{}", response.text());
                let next = response.browser(&browser);
                let target = redirect.unwrap_or_else(|| page.to_owned());
                let props = app.page(&next, &target).await;
                props.assert_status(200);
                let props = props.json();
                assert_json_eq(&after["flash"], &props["flash"]);
                assert_json_eq(&after["auth_user"], &props["auth"]["user"]);
                if !after["active_character"].is_null() {
                    assert_eq!(
                        after["active_character"],
                        props["auth"]["active_character"]["id"]
                    );
                }
            }
        }
        _ => {
            response.assert_status(status);
            let (_, _, message) = error_page_text(golden["html"].as_str().expect("html"));
            let expected = if status == 404 {
                response.json()["message"]
                    .as_str()
                    .expect("a message")
                    .starts_with("No query results for model")
            } else {
                response.json()["message"] == message.as_str()
            };
            assert!(expected, "{}: {}", message, response.text());
        }
    }

    let mut rows = golden["rows"].clone();
    if golden["method"] == "DELETE" && path.starts_with("/auth/character/") && status == 302 {
        // A removed character stops syncing (FIXES.md, "Removed
        // characters were mailed about lapsed tokens forever").
        let removed: i64 = path
            .rsplit('/')
            .next()
            .and_then(|id| id.parse().ok())
            .expect("a character id");
        for character in rows["characters"].as_array_mut().expect("characters") {
            if character["id"] == removed {
                character["should_sync"] = Value::Bool(false);
            }
        }
    }

    assert_json_eq(&golden["jobs"], &golden::jobs(&app).await);
    if golden["fakes"]
        .as_object()
        .is_some_and(|fakes| !fakes.is_empty())
    {
        let expected = golden::expected_calls(&golden, &app.config.esi.user_agent);
        assert_json_eq(&expected, &golden::comparable_calls(&app));
    }
    assert_rows_eq(&rows, &golden::rows(&app).await, &app);
}

macro_rules! golden_mutations {
    ($($name:ident),* $(,)?) => {
        $(
            #[tokio::test]
            async fn $name() {
                check(stringify!($name)).await;
            }
        )*

        const TESTED: &[&str] = &[$(stringify!($name)),*];
    };
}

golden_mutations!(
    admin_discord,
    admin_discord_clear,
    admin_discord_first,
    admin_discord_forbidden,
    admin_discord_http,
    admin_discord_invalid,
    admin_discord_number,
    admin_resolve_done,
    admin_resolve_forbidden,
    admin_resolve_invalid,
    admin_resolve_rejected,
    admin_resolve_unknown,
    admin_source_character,
    admin_source_corporation,
    admin_source_forbidden,
    admin_source_invalid,
    admin_source_missing,
    admin_source_no_alliance,
    admin_source_same,
    admin_sync_now,
    logout,
    remove_active_main,
    remove_alt,
    remove_foreign,
    remove_last,
    request_again_rejected,
    request_character,
    request_corporation,
    request_done,
    request_foreign,
    request_has_standing,
    request_invalid_type,
    request_no_alliance,
    request_no_source,
    request_pending,
    request_source_entity,
    set_main,
    set_main_completes_setup,
    set_main_foreign,
    settings_put,
    setup_complete,
    switch_character,
    switch_foreign,
    sync_now,
    sync_now_covered,
    sync_now_nothing,
    sync_off,
    sync_on,
    sync_toggle_foreign,
    sync_toggle_invalid,
    sync_toggle_missing,
    sync_toggle_string,
    sync_toggle_unknown,
);

/// A guest's logout was sent to the login by Laravel (once the CSRF check
/// passed); the form now answers 401 with the login (FIXES.md, "Cross-site
/// forms are refused without a CSRF token").
#[tokio::test]
async fn logout_guest() {
    let golden = golden::load("mutations", "logout_guest");
    assert_eq!(golden["status"], 302);
    let app = golden::app(&Value::Null).await;
    let response = app
        .post(&Browser::guest(), "/logout", serde_json::json!({}))
        .await;
    response.assert_status(401);
    assert_eq!(response.json()["redirect"], "/eve");
}

#[test]
fn every_recorded_form_has_a_test() {
    let mut tested: Vec<String> = TESTED.iter().map(|name| (*name).to_owned()).collect();
    tested.push("logout_guest".to_owned());
    tested.sort();
    assert_eq!(golden::names("mutations"), tested);
}

//! Every page GET the Laravel reference answered
//! (tests/fixtures/golden/pages): the props of a page, its redirect, or
//! the error page, for guests, pilots and the admin.

use serde_json::Value;

use crate::common::golden::{self, error_page_text, page_props};
use crate::common::{APP_URL, Browser, TestApp, TestResponse, assert_json_eq};

/// The paths the api answers itself; every other path is a page whose
/// props come from `/api/pages/*`.
fn served_by_api(path: &str) -> bool {
    path == "/eve"
        || path.starts_with("/eve?")
        || path.starts_with("/eve/")
        || path == "/up"
        || path == "/settings"
        || path.starts_with("/storage/")
        || path == "/nope"
}

async fn browser(app: &TestApp, golden: &Value) -> Browser {
    match (golden["user"].as_i64(), golden["character"].as_i64()) {
        (Some(user), Some(character)) => app.sign_in_as(user, character).await,
        (Some(user), None) => app.sign_in(user).await,
        _ => Browser::guest(),
    }
}

/// N3: the login redirect without its random state, which must be 40
/// letters and digits.
fn without_state(location: &str) -> String {
    let (before, state) = location.split_once("&state=").expect("a state");
    assert_eq!(state.len(), 40, "{location}");
    assert!(
        state.chars().all(|c| c.is_ascii_alphanumeric()),
        "{location}"
    );
    before.to_owned()
}

/// The tickers Laravel lost on the admin settings page (FIXES.md, "The
/// admin settings page lost the active character's tickers"): the
/// dataset's.
fn with_tickers(props: &mut Value) {
    let dataset: Value = serde_json::from_str(
        &std::fs::read_to_string(crate::common::repo_path("tests/fixtures/dataset.json"))
            .expect("the dataset"),
    )
    .expect("dataset JSON");
    let ticker = |table: &str, id: &Value| {
        dataset[table]
            .as_array()
            .and_then(|rows| rows.iter().find(|row| &row["id"] == id))
            .map(|row| row["ticker"].clone())
            .unwrap_or(Value::Null)
    };
    let fill = |character: &mut Value| {
        for (key, table) in [("corporation", "corporations"), ("alliance", "alliances")] {
            if character[key].is_object() && character[key]["ticker"].is_null() {
                let id = character[key]["id"].clone();
                character[key]["ticker"] = ticker(table, &id);
            }
        }
    };
    fill(&mut props["auth"]["active_character"]);
    for character in props["auth"]["characters"]
        .as_array_mut()
        .into_iter()
        .flatten()
    {
        fill(character);
    }
}

async fn check(name: &str) {
    let golden = golden::load("pages", name);
    let app = golden::app(&golden["sql"]).await;
    let browser = browser(&app, &golden).await;
    let path = golden["path"].as_str().expect("a path");
    let status = golden["status"].as_u64().expect("a status") as u16;
    let response: TestResponse = if served_by_api(path) {
        app.get_with(&browser, path, &[("accept", "text/html")])
            .await
    } else {
        app.page(&browser, path).await
    };

    match status {
        200 if golden.get("props").is_some() => {
            response.assert_status(200);
            let mut expected = page_props(&golden);
            if name.starts_with("admin_settings") {
                with_tickers(&mut expected);
            }
            if name == "admin_settings_alt" {
                // The source options and the mail scope are the admin
                // character's, not the alt's (FIXES.md, "The source came
                // from the active character, not an admin character").
                let admin = page_props(&golden::load("pages", "admin_settings"));
                for key in ["sourceTypes", "adminCharacter"] {
                    expected[key] = admin[key].clone();
                }
            }
            let mut actual = response.json();
            golden::unmock(&mut actual, &app);
            assert_json_eq(&expected, &actual);
        }
        200 => {
            // Laravel's health page.
            response.assert_status(200);
            let html = golden["html"].as_str().expect("html");
            assert!(html.contains("Application up"));
            assert!(
                response.text().contains("Application up"),
                "{}",
                response.text()
            );
            assert!(response.text().contains("<title>Bluebook</title>"));
        }
        302 => {
            response.assert_status(302);
            let expected = golden["location"].as_str().expect("a location");
            if expected.contains("&state=") {
                // The SSO is the mock's (N5).
                let actual = response
                    .location()
                    .replace(&app.sso.url(), "https://login.eveonline.com");
                assert_eq!(without_state(expected), without_state(&actual));
            } else {
                assert_eq!(response.location(), expected);
            }
        }
        500 => {
            // An unknown scope was a server error (FIXES.md, "An unknown
            // scope in the login link was a server error").
            assert_eq!(name, "login_unknown_scope");
            response.assert_status(400);
            let (title, code, message) = error_page_text(&response.text());
            assert_eq!(
                (title.as_str(), code.as_str(), message.as_str()),
                ("Bad Request", "400", "Bad Request")
            );
        }
        _ => {
            response.assert_status(status);
            let expected = error_page_text(golden["html"].as_str().expect("html"));
            if served_by_api(path) {
                assert_eq!(error_page_text(&response.text()), expected);
            } else {
                // The page's props answer the message; SvelteKit's error
                // page shows it (frontend/e2e/routes.spec.ts).
                assert_eq!(response.json()["message"], expected.2.as_str());
            }
        }
    }
    assert!(APP_URL.starts_with("https://"));
}

macro_rules! golden_pages {
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

golden_pages!(
    admin_guest,
    admin_overview,
    admin_overview_no_source,
    admin_overview_unreadable,
    admin_pilot,
    admin_pilots,
    admin_pilots_alliances,
    admin_pilots_alliances_ticker,
    admin_pilots_corporations,
    admin_pilots_corporations_member,
    admin_pilots_corporations_page_2,
    admin_pilots_corporations_search,
    admin_pilots_pilot,
    admin_pilots_search,
    admin_pilots_unknown_view,
    admin_requests,
    admin_requests_done,
    admin_requests_page_2,
    admin_requests_rejected,
    admin_requests_unknown_status,
    admin_settings,
    admin_settings_alt,
    admin_settings_no_source,
    appearance_guest,
    appearance_pilot,
    callback_without_state,
    dashboard_admin,
    dashboard_character_source,
    dashboard_fresh,
    dashboard_guest,
    dashboard_member,
    dashboard_no_source,
    dashboard_pilot,
    dashboard_pilot_alt,
    dashboard_pilot_no_active,
    dashboard_red,
    home_guest,
    home_pilot,
    login,
    login_scopes,
    login_unknown_scope,
    not_found,
    settings_guest,
    settings_pilot,
    storage,
    up,
);

#[test]
fn every_recorded_page_has_a_test() {
    let mut tested: Vec<String> = TESTED.iter().map(|name| (*name).to_owned()).collect();
    tested.sort();
    assert_eq!(golden::names("pages"), tested);
}

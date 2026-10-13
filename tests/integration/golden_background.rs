//! The scheduled commands and the jobs the Laravel reference ran
//! (tests/fixtures/golden/background): exit codes, queued jobs, every call
//! to ESI, EVE SSO and Discord, and the rows they left behind.

use serde_json::{Value, json};

use bluebook::jobs::Job;

use crate::common::golden::{self, assert_rows_eq};
use crate::common::{APP_URL, TestApp, assert_json_eq};

/// The admin grant link the "source unreadable" alert carries now
/// (FIXES.md, "The "source unreadable" alert linked a login without
/// scopes").
fn admin_grant_link() -> String {
    format!(
        "{APP_URL}/eve?add_to_account=1&scopes=publicData%2Cesi-characters.read_contacts.v1%2Cesi-characters.write_contacts.v1%2Cesi-corporations.read_contacts.v1%2Cesi-alliances.read_contacts.v1%2Cesi-mail.send_mail.v1"
    )
}

/// The recorded calls with the alert's link fixed.
fn with_fixed_alert(mut calls: Value) -> Value {
    let plain = format!("{APP_URL}/eve");
    for call in calls["discord"].as_array_mut().expect("discord") {
        let Some(embed) = call["body"]["embeds"][0].as_object_mut() else {
            continue;
        };
        if embed.get("title") == Some(&json!("Standings source unreadable")) {
            let description = embed["description"].as_str().expect("a description");
            embed.insert(
                "description".to_owned(),
                json!(
                    description
                        .replace(&format!("({plain})"), &format!("({})", admin_grant_link()))
                ),
            );
            embed.insert("url".to_owned(), json!(admin_grant_link()));
        }
    }
    calls
}

/// A step of record.py: an artisan command, the queue worker, or a job
/// dispatched from tinker.
async fn run_step(app: &TestApp, step: &Value) {
    if let Some(command) = step["artisan"].as_array() {
        let args: Vec<&str> = command.iter().filter_map(Value::as_str).collect();
        let code = match args.as_slice() {
            ["standings:sync"] => bluebook::standings::sync::run(&app.state, false).await,
            ["standings:sync", "--force"] => bluebook::standings::sync::run(&app.state, true).await,
            ["characters:sync-affiliations"] => {
                bluebook::entities::sync_affiliations(&app.state).await
            }
            ["standings:check-tokens"] => bluebook::standings::tokens::check(&app.state).await,
            other => panic!("no command {other:?}"),
        };
        assert_eq!(json!(code), step["exit"], "{args:?}");
    } else if step["queue"] == json!(true) {
        bluebook::jobs::run_pending(&app.state).await;
        assert_eq!(json!(golden::failed_jobs(app).await), step["failed"]);
    } else if let Some(code) = step["php"].as_str() {
        let id = |prefix: &str| -> i64 {
            let start = code.find(prefix).expect("a find") + prefix.len();
            code[start..]
                .split(')')
                .next()
                .and_then(|id| id.parse().ok())
                .expect("an id")
        };
        if code.starts_with("App\\Jobs\\SyncCharacterStandings::dispatchSync") {
            bluebook::standings::character::sync(&app.state, id("Character::find("))
                .await
                .expect("a character sync");
        } else if code.starts_with("App\\Jobs\\NotifyStandingRequest::dispatch") {
            let standing_request_id = id("StandingRequest::find(");
            bluebook::jobs::enqueue(
                &app.state,
                &Job::DiscordRequest {
                    standing_request_id,
                },
            )
            .await
            .expect("a queued job");
        } else {
            panic!("no job for {code}");
        }
    }
    assert_json_eq(&step["jobs"], &golden::jobs(app).await);
}

/// The recorded outcome with the token and ledger rules of FIXES.md
/// ("Tokens were thrown away on any failure", "A failed read of a
/// character's contacts counted as no contacts").
fn with_fixed_tokens(name: &str, golden: &mut Value) {
    match name {
        // A 403 is a missing in-game role: the reader's token stays, as in
        // the run where nothing went wrong.
        "sync_forbidden" => {
            golden["rows"]["esi_tokens"] =
                golden::load("background", "sync_unchanged")["rows"]["esi_tokens"].clone();
        }
        // The read of the character's contacts failed: the sync stops
        // there, so no writes and no ledger rows for it.
        "character_sync_unreadable" => {
            golden["http"]
                .as_array_mut()
                .expect("calls")
                .retain(|call| call["method"] == "GET");
            golden["rows"]["character_synced_contacts"]
                .as_array_mut()
                .expect("ledger rows")
                .retain(|row| row["character_id"] != json!(2_112_000_002));
        }
        _ => {}
    }
}

async fn check(name: &str) {
    let mut golden = golden::load("background", name);
    with_fixed_tokens(name, &mut golden);
    let app = golden::app(&golden["sql"]).await;
    golden::configure_fakes(&app, &golden["fakes"]);
    for step in golden["steps"].as_array().expect("steps") {
        run_step(&app, step).await;
    }
    let expected = golden::expected_calls(&golden, &app.config.esi.user_agent);
    assert_json_eq(&with_fixed_alert(expected), &golden::comparable_calls(&app));
    assert_rows_eq(&golden["rows"], &golden::rows(&app).await, &app);
}

macro_rules! golden_background {
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

golden_background!(
    affiliations,
    character_sync,
    character_sync_refresh,
    character_sync_unreadable,
    character_sync_write_only,
    check_tokens,
    check_tokens_no_mail_scope,
    notify_request,
    notify_request_alliance,
    notify_request_env_webhook,
    sync_changed,
    sync_forbidden,
    sync_no_source,
    sync_unchanged,
    sync_unchanged_forced,
    sync_unreadable,
    sync_unreadable_corporation,
);

#[test]
fn every_recorded_run_has_a_test() {
    let mut tested: Vec<String> = TESTED.iter().map(|name| (*name).to_owned()).collect();
    tested.sort();
    assert_eq!(golden::names("background"), tested);
}

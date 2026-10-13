//! The scheduled tasks and jobs in the world (common::world): every call
//! they make to ESI, EVE SSO and Discord, and what they leave behind.

use serde_json::{Value, json};

use bluebook::jobs::Job;

use crate::common::world::{NICOLAS, RED, TINA};
use crate::common::{APP_URL, TestApp, assert_json_eq};

/// The ESI calls as `METHOD /path?query` with their JSON body.
fn esi_calls(app: &TestApp) -> Vec<(String, Value)> {
    app.esi
        .requests()
        .into_iter()
        .map(|call| {
            let body = if call.body.is_empty() {
                Value::Null
            } else {
                call.json()
            };
            (call.line(), body)
        })
        .collect()
}

async fn ledger(app: &TestApp, character: i64) -> Vec<Value> {
    app.rows(&format!(
        "select contact_id, standing::float8 as standing from character_synced_contacts
         where character_id = {character} order by contact_id"
    ))
    .await
}

const ADMIN_GRANT: &str = "/eve?add_to_account=1&scopes=publicData%2Cesi-characters.read_contacts.v1%2Cesi-characters.write_contacts.v1%2Cesi-corporations.read_contacts.v1%2Cesi-alliances.read_contacts.v1%2Cesi-mail.send_mail.v1";

/// A token about to expire is refreshed before the contacts are read;
/// then the missing contacts are added by standing, changed ones edited
/// and the synced contact the source dropped deleted. Personal contacts
/// stay.
#[tokio::test]
async fn a_character_sync_refreshes_the_token_and_writes_the_difference() {
    let app = TestApp::world().await;
    app.exec(&format!(
        "update esi_tokens set expires_at = '2026-06-25T14:02:00Z' where character_id = {NICOLAS}"
    ))
    .await;
    app.sso.refresh_answer(json!({
        "access_token": "new-access-token",
        "expires_in": 1199,
        "token_type": "Bearer",
        "refresh_token": "new-refresh-token"
    }));
    app.esi.contacts(
        "characters/95000001",
        &[
            (99_000_001, "alliance", 10.0),
            (98_000_002, "corporation", 5.0),
            (12_345, "character", 5.0),
            (55_555, "character", -10.0),
        ],
    );

    bluebook::standings::character::sync(&app.state, NICOLAS)
        .await
        .expect("a character sync");

    let refreshes = app.sso.requests();
    assert_eq!(refreshes.len(), 1);
    assert_eq!(
        refreshes[0].text(),
        "grant_type=refresh_token&refresh_token=refresh-token-3"
    );
    for call in app.esi.requests() {
        assert_eq!(
            call.header("authorization"),
            Some("Bearer new-access-token")
        );
    }
    assert_eq!(
        esi_calls(&app),
        [
            (
                "GET /characters/95000001/contacts/?page=1".to_owned(),
                Value::Null
            ),
            (
                "POST /characters/95000001/contacts/?standing=5&watched=0".to_owned(),
                json!([95000100, 98000099])
            ),
            (
                "POST /characters/95000001/contacts/?standing=0&watched=0".to_owned(),
                json!([95000002])
            ),
            (
                "POST /characters/95000001/contacts/?standing=-5&watched=0".to_owned(),
                json!([98000003, 500001])
            ),
            (
                "POST /characters/95000001/contacts/?standing=-10&watched=0".to_owned(),
                json!([99000003, 95000101, 98000004])
            ),
            (
                "PUT /characters/95000001/contacts/?standing=10&watched=0".to_owned(),
                json!([98000002])
            ),
            (
                "DELETE /characters/95000001/contacts/?contact_ids=12345".to_owned(),
                Value::Null
            ),
        ]
    );
    assert_json_eq(
        &json!([
            { "contact_id": 500001, "standing": -5 },
            { "contact_id": 95000002, "standing": 0 },
            { "contact_id": 95000100, "standing": 5 },
            { "contact_id": 95000101, "standing": -10 },
            { "contact_id": 98000002, "standing": 10 },
            { "contact_id": 98000003, "standing": -5 },
            { "contact_id": 98000004, "standing": -10 },
            { "contact_id": 98000099, "standing": 5 },
            { "contact_id": 99000001, "standing": 10 },
            { "contact_id": 99000003, "standing": -10 }
        ]),
        &json!(ledger(&app, NICOLAS).await),
    );
    assert_eq!(
        app.scalar(&format!(
            "select access_token from esi_tokens where character_id = {NICOLAS}"
        ))
        .await,
        "new-access-token"
    );
}

/// Without the read scope nothing is read: every source contact is
/// written, grouped by standing.
#[tokio::test]
async fn a_character_with_only_the_write_scope_gets_every_contact_written() {
    let app = TestApp::world().await;

    bluebook::standings::character::sync(&app.state, RED)
        .await
        .expect("a character sync");

    assert_eq!(
        esi_calls(&app),
        [
            (
                "POST /characters/95000020/contacts/?standing=10&watched=0".to_owned(),
                json!([99000001, 98000002])
            ),
            (
                "POST /characters/95000020/contacts/?standing=5&watched=0".to_owned(),
                json!([95000100, 98000099])
            ),
            (
                "POST /characters/95000020/contacts/?standing=0&watched=0".to_owned(),
                json!([95000002])
            ),
            (
                "POST /characters/95000020/contacts/?standing=-5&watched=0".to_owned(),
                json!([98000003, 500001])
            ),
            (
                "POST /characters/95000020/contacts/?standing=-10&watched=0".to_owned(),
                json!([99000003, 95000101, 98000004])
            ),
        ]
    );
    assert_eq!(ledger(&app, RED).await.len(), 10);
}

/// An alliance request is announced with the alliance's logo; without a
/// role nobody is pinged.
#[tokio::test]
async fn an_alliance_request_is_announced() {
    let app = TestApp::world().await;
    app.exec("update discord_settings set role_id = null").await;

    bluebook::jobs::enqueue(
        &app.state,
        &Job::DiscordRequest {
            standing_request_id: 3,
        },
    )
    .await
    .expect("a queued announcement");
    bluebook::jobs::run_pending(&app.state).await;

    let requests = app.discord.requests();
    assert_eq!(requests.len(), 1);
    assert_json_eq(
        &json!({
            "content": "A alliance requested standings.",
            "embeds": [{
                "title": "Red Alliance",
                "description": "Requested by Red Pilot. Review it in the administration page.\n```\nRed Alliance\n```",
                "url": format!("{APP_URL}/admin"),
                "thumbnail": {
                    "url": "https://images.evetech.net/alliances/99000003/logo?size=128"
                },
                "fields": [
                    { "name": "Type", "value": "Alliance", "inline": true },
                    { "name": "Requested by", "value": "Red Pilot", "inline": true }
                ]
            }]
        }),
        &requests[0].json(),
    );
    assert!(app.jobs().await.is_empty());
}

/// A source no admin can read fails the run and alerts Discord once an
/// hour, naming the source and linking the admin grant.
#[tokio::test]
async fn an_unreadable_source_alerts_discord_once() {
    let app = TestApp::world().await;
    app.exec("delete from esi_tokens where character_id = 2112000001")
        .await;

    assert_eq!(bluebook::standings::sync::run(&app.state, false).await, 1);
    assert_eq!(bluebook::standings::sync::run(&app.state, false).await, 1);
    assert_json_eq(
        &json!([["discord_unreadable", {}]]),
        &json!(app.jobs().await),
    );
    bluebook::jobs::run_pending(&app.state).await;

    assert!(app.esi.requests().is_empty());
    let requests = app.discord.requests();
    assert_eq!(requests.len(), 1);
    let grant = format!("{APP_URL}{ADMIN_GRANT}");
    assert_json_eq(
        &json!({
            "content": "<@&777> :warning: The standings source could not be read.",
            "embeds": [{
                "title": "Standings source unreadable",
                "description": format!("Couldn't read **Home Alliance**'s contacts. An admin character's ESI token may be missing or lacking permissions.\n\n**[Click here to re-authenticate]({grant})**"),
                "url": grant
            }],
            "allowed_mentions": { "roles": ["777"] }
        }),
        &requests[0].json(),
    );
}

/// The admin cannot read a corporation they are not in: the read fails
/// and the alert names the corporation.
#[tokio::test]
async fn an_unreadable_corporation_source_names_the_corporation() {
    let app = TestApp::world().await;
    app.exec("update standings_sources set type = 'corporation', entity_id = 98000001")
        .await;
    app.exec("update discord_settings set role_id = null").await;
    app.esi.fail_path("/corporations/98000001/contacts/", 403);

    assert_eq!(bluebook::standings::sync::run(&app.state, false).await, 1);
    bluebook::jobs::run_pending(&app.state).await;

    assert_eq!(
        app.esi
            .requests()
            .iter()
            .map(|call| call.line())
            .collect::<Vec<_>>(),
        ["GET /corporations/98000001/contacts/?page=1"]
    );
    let grant = format!("{APP_URL}{ADMIN_GRANT}");
    assert_json_eq(
        &json!({
            "content": ":warning: The standings source could not be read.",
            "embeds": [{
                "title": "Standings source unreadable",
                "description": format!("Couldn't read **Home Corp**'s contacts. An admin character's ESI token may be missing or lacking permissions.\n\n**[Click here to re-authenticate]({grant})**"),
                "url": grant
            }]
        }),
        &app.discord.requests()[0].json(),
    );
}

/// Tina Pilot syncs without a write token: she is mailed once by the
/// admin character, however often the check runs.
#[tokio::test]
async fn the_token_check_mails_a_lapsed_pilot_once() {
    let app = TestApp::world().await;

    assert_eq!(bluebook::standings::tokens::check(&app.state).await, 0);
    assert_eq!(bluebook::standings::tokens::check(&app.state).await, 0);
    assert_json_eq(
        &json!([["token_mail", { "sender_id": 2112000001, "recipient_id": TINA }]]),
        &json!(app.jobs().await),
    );
    bluebook::jobs::run_pending(&app.state).await;

    assert_eq!(
        esi_calls(&app),
        [(
            "POST /characters/2112000001/mail/".to_owned(),
            json!({
                "approved_cost": 0,
                "body": format!("Hello Tina Pilot,\n\nYour contact permissions for Bluebook have lapsed, so your standings can't be synced. Please grant them again here:\n{APP_URL}/eve?add_to_account=1&scopes=esi-characters.read_contacts.v1%2Cesi-characters.write_contacts.v1\n\nso the service won't get interrupted."),
                "recipients": [{ "recipient_id": TINA, "recipient_type": "character" }],
                "subject": "Bluebook \u{2014} re-authentication required"
            })
        )]
    );
}

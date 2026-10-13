//! The EVE login end to end against the mock SSO and ESI: the
//! character's corporation and alliance stored on login, a character
//! moving to another account, and the callback's redirects, flashes,
//! tokens and session.

use serde_json::json;

use crate::common::factories::{NewCharacter, Owner};
use crate::common::mocks::esi_corporation_payload;
use crate::common::{
    APP_URL, Browser, READ_CHARACTER_CONTACTS, TestApp, WRITE_CHARACTER_CONTACTS, assert_json_eq,
};

fn corporation_response(alliance_id: Option<i64>) -> serde_json::Value {
    let mut corporation = json!({
        "ceo_id": 95_000_002,
        "creator_id": 95_000_003,
        "date_founded": "2020-01-01T00:00:00Z",
        "description": "A test corporation.",
        "faction_id": null,
        "home_station_id": null,
        "member_count": 42,
        "name": "Test Corporation",
        "shares": 1000,
        "tax_rate": 0.1,
        "ticker": "TSTC",
        "url": "https://example.com",
        "war_eligible": true,
    });
    if let Some(alliance_id) = alliance_id {
        corporation["alliance_id"] = json!(alliance_id);
    }
    corporation
}

fn alliance_response() -> serde_json::Value {
    json!({
        "creator_corporation_id": 98_000_002,
        "creator_id": 95_000_004,
        "date_founded": "2019-01-01T00:00:00Z",
        "name": "Test Alliance",
        "ticker": "TSTA",
        "executor_corporation_id": 98_000_001,
        "faction_id": null,
    })
}

/// The character the SSO signs in and its corporation, as ESI knows them.
fn esi_knows(app: &TestApp, character: i64, corporation: i64) {
    app.esi.affiliation(character, corporation, None);
    app.esi
        .corporation_json(corporation, esi_corporation_payload("Login Corp", None));
}

#[tokio::test]
async fn fetches_and_stores_corporation_and_alliance_details_on_login() {
    let app = TestApp::new().await;
    app.sso
        .sign_in_as(95_000_001, "Test Pilot", "owner-hash", &[]);
    app.esi
        .affiliation(95_000_001, 98_000_001, Some(99_000_001));
    app.esi
        .corporation_json(98_000_001, corporation_response(Some(99_000_001)));
    app.esi.alliance_json(99_000_001, alliance_response());

    app.eve_login(&Browser::guest(), "").await;

    assert_json_eq(
        &json!([{ "corporation_id": 98_000_001, "alliance_id": 99_000_001 }]),
        &json!(
            app.rows("select corporation_id, alliance_id from characters where id = 95000001")
                .await
        ),
    );
    assert_json_eq(
        &json!([{ "name": "Test Corporation", "ticker": "TSTC", "member_count": 42 }]),
        &json!(
            app.rows("select name, ticker, member_count from corporations where id = 98000001")
                .await
        ),
    );
    assert_json_eq(
        &json!([{ "name": "Test Alliance", "ticker": "TSTA" }]),
        &json!(
            app.rows("select name, ticker from alliances where id = 99000001")
                .await
        ),
    );
}

#[tokio::test]
async fn stores_a_character_with_no_alliance() {
    let app = TestApp::new().await;
    app.sso
        .sign_in_as(95_000_001, "Test Pilot", "owner-hash", &[]);
    app.esi.affiliation(95_000_001, 98_000_001, None);
    app.esi
        .corporation_json(98_000_001, corporation_response(None));

    app.eve_login(&Browser::guest(), "").await;

    assert_json_eq(
        &json!([{ "alliance_id": null }]),
        &json!(
            app.rows("select alliance_id from characters where id = 95000001")
                .await
        ),
    );
    assert_json_eq(
        &json!([{ "name": "Test Corporation" }]),
        &json!(
            app.rows("select name from corporations where id = 98000001")
                .await
        ),
    );
    assert_eq!(app.count("alliances").await, 0);
}

// Adding a character another account owns, through the login with
// `add_to_account`.

/// The new owner logs in with the character, adding it to the account.
async fn add_to_account(app: &TestApp, new_owner: i64, character: i64) {
    esi_knows(app, character, 98_000_001);
    app.sso
        .sign_in_as(character, "Transferred", "another-owner", &[]);
    let browser = app.sign_in(new_owner).await;
    let (_, callback) = app.eve_login(&browser, "?add_to_account=1").await;
    callback.assert_status(302);
}

#[tokio::test]
async fn adding_another_users_character_clears_their_main_selection() {
    let app = TestApp::new().await;
    let previous = app.user().await;
    app.character_of(previous).await;
    let transferred = app.character_of(previous).await;
    app.set_main(previous, transferred).await;
    let new_owner = app.user().await;
    app.character_of(new_owner).await;

    add_to_account(&app, new_owner, transferred).await;

    assert_eq!(app.owner_of(transferred).await, Some(new_owner));
    assert_eq!(app.main_character_of(previous).await, None);
}

#[tokio::test]
async fn adding_another_users_character_keeps_an_unrelated_main_selection() {
    let app = TestApp::new().await;
    let previous = app.user().await;
    let main = app.character_of(previous).await;
    let transferred = app.character_of(previous).await;
    app.set_main(previous, main).await;
    let new_owner = app.user().await;
    app.character_of(new_owner).await;

    add_to_account(&app, new_owner, transferred).await;

    assert_eq!(app.main_character_of(previous).await, Some(main));
}

#[tokio::test]
async fn taking_a_users_last_character_deletes_the_abandoned_account() {
    let app = TestApp::new().await;
    let previous = app.user().await;
    let transferred = app.character_of(previous).await;
    app.set_main(previous, transferred).await;
    let new_owner = app.user().await;
    app.character_of(new_owner).await;

    add_to_account(&app, new_owner, transferred).await;

    assert_eq!(app.owner_of(transferred).await, Some(new_owner));
    assert!(!app.user_exists(previous).await);
}

#[tokio::test]
async fn re_adding_your_own_character_changes_nothing() {
    let app = TestApp::new().await;
    let owner = app.user().await;
    let main = app.character_of(owner).await;
    app.set_main(owner, main).await;

    add_to_account(&app, owner, main).await;

    assert_eq!(app.main_character_of(owner).await, Some(main));
    assert!(app.user_exists(owner).await);
}

// The callback.

#[tokio::test]
async fn a_login_starts_a_session() {
    let app = TestApp::new().await;
    app.sso
        .sign_in_as(95_000_001, "Test Pilot", "owner-hash", &[]);
    esi_knows(&app, 95_000_001, 98_000_001);

    let (browser, callback) = app.eve_login(&Browser::guest(), "").await;

    callback.assert_status(302);
    assert_eq!(callback.location(), format!("{APP_URL}/dashboard"));
    let cookie = callback
        .set_cookies()
        .into_iter()
        .find(|cookie| cookie.starts_with("bluebook-session="))
        .expect("a session cookie");
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    assert!(cookie.contains("SameSite=Lax"), "{cookie}");
    assert!(cookie.contains("Secure"), "{cookie}");
    assert!(cookie.contains("Max-Age=34560000"), "{cookie}");

    let props = app.page(&browser, "/dashboard").await;
    props.assert_status(200);
    let props = props.json();
    assert_eq!(props["auth"]["user"]["name"], "Test Pilot");
    assert_eq!(props["auth"]["active_character"]["id"], 95_000_001);
    assert_json_eq(
        &json!({ "success": "Welcome back, Test Pilot!", "error": null, "info": null }),
        &props["flash"],
    );
    assert_json_eq(
        &json!([{ "name": "Test Pilot", "user_id": 1, "should_sync": false }]),
        &json!(
            app.rows("select name, user_id, should_sync from characters")
                .await
        ),
    );
}

#[tokio::test]
async fn the_login_returns_to_the_page_a_guest_was_sent_away_from() {
    let app = TestApp::new().await;
    app.sso
        .sign_in_as(95_000_001, "Test Pilot", "owner-hash", &[]);
    esi_knows(&app, 95_000_001, 98_000_001);

    let away = app
        .page(&Browser::guest(), "/admin/pilots?view=alliances")
        .await;
    away.assert_status(302);
    assert_eq!(away.location(), format!("{APP_URL}/eve"));
    let browser = away.browser(&Browser::guest());

    let (_, callback) = app.eve_login(&browser, "").await;
    assert_eq!(
        callback.location(),
        format!("{APP_URL}/admin/pilots?view=alliances")
    );
}

#[tokio::test]
async fn adding_a_character_flashes_and_goes_to_the_dashboard() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    app.sso.sign_in_as(95_000_009, "New Alt", "alt-owner", &[]);
    esi_knows(&app, 95_000_009, 98_000_001);

    let (browser, callback) = app
        .eve_login(&app.sign_in(user).await, "?add_to_account=1")
        .await;

    assert_eq!(callback.location(), format!("{APP_URL}/dashboard"));
    assert_eq!(app.owner_of(95_000_009).await, Some(user));
    let props = app.page(&browser, "/dashboard").await.json();
    assert_eq!(
        props["flash"]["success"],
        "Your character has been added to your account."
    );
    assert_eq!(props["auth"]["active_character"]["id"], 95_000_009);
}

#[tokio::test]
async fn a_login_with_scopes_stores_its_token() {
    let app = TestApp::new().await;
    app.sso.sign_in_as(
        95_000_001,
        "Test Pilot",
        "owner-hash",
        &[
            READ_CHARACTER_CONTACTS,
            WRITE_CHARACTER_CONTACTS,
            "esi-nope.unknown.v1",
        ],
    );
    esi_knows(&app, 95_000_001, 98_000_001);

    app.eve_login(&Browser::guest(), "").await;

    assert_json_eq(
        &json!([{
            "character_id": 95_000_001,
            "refresh_token": "test-refresh-token",
            "token_type": "Bearer",
            "character_owner_hash": "owner-hash",
            "scopes": [READ_CHARACTER_CONTACTS, WRITE_CHARACTER_CONTACTS],
            "expires_at": "2026-06-25T14:19:59+00:00",
        }]),
        &json!(app.rows(
            "select character_id, refresh_token, token_type, character_owner_hash, scopes,
                to_char(expires_at at time zone 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"+00:00\"') as expires_at
             from esi_tokens"
        ).await),
    );
}

#[tokio::test]
async fn a_login_without_scopes_stores_no_token() {
    let app = TestApp::new().await;
    app.sso
        .sign_in_as(95_000_001, "Test Pilot", "owner-hash", &[]);
    esi_knows(&app, 95_000_001, 98_000_001);

    app.eve_login(&Browser::guest(), "").await;

    assert_eq!(app.count("characters").await, 1);
    assert_eq!(app.count("esi_tokens").await, 0);
}

#[tokio::test]
async fn a_character_on_a_new_eve_account_gets_a_new_user() {
    let app = TestApp::new().await;
    let old_user = app.user().await;
    let character = app
        .character(NewCharacter {
            id: Some(95_000_001),
            owner: Owner::User(old_user),
            ..NewCharacter::default()
        })
        .await;
    app.sso
        .sign_in_as(character, "Sold Pilot", "the-buyers-hash", &[]);
    esi_knows(&app, character, 98_000_001);

    let (browser, _) = app.eve_login(&Browser::guest(), "").await;

    let new_owner = app.owner_of(character).await.expect("an owner");
    assert_ne!(new_owner, old_user);
    assert!(!app.user_exists(old_user).await);
    let props = app.page(&browser, "/dashboard").await.json();
    assert_eq!(props["auth"]["user"]["name"], "Sold Pilot");
}

#[tokio::test]
async fn the_callback_without_a_valid_state_goes_home_with_an_error() {
    let app = TestApp::new().await;
    app.sso
        .sign_in_as(95_000_001, "Test Pilot", "owner-hash", &[]);
    esi_knows(&app, 95_000_001, 98_000_001);

    let callback = app
        .get(
            &Browser::guest(),
            "/eve/callback?code=valid-code&state=forged",
        )
        .await;

    callback.assert_status(302);
    assert_eq!(callback.location(), APP_URL);
    let props = app
        .page(&callback.browser(&Browser::guest()), "/")
        .await
        .json();
    assert_eq!(
        props["flash"]["error"],
        "Unable to retrieve user information from EVE Online. Please try again later."
    );
    assert_eq!(app.count("users").await, 0);
}

#[tokio::test]
async fn a_failed_affiliation_lookup_goes_home_with_an_error() {
    let app = TestApp::new().await;
    app.sso
        .sign_in_as(95_000_001, "Test Pilot", "owner-hash", &[]);
    app.esi.fail_path("/characters/affiliation/", 400);

    let (browser, callback) = app.eve_login(&Browser::guest(), "").await;

    assert_eq!(callback.location(), APP_URL);
    let props = app.page(&browser, "/").await.json();
    assert_eq!(
        props["flash"]["error"],
        "Unable to retrieve user information from EVE Online. Please try again later."
    );
    assert_eq!(app.count("characters").await, 0);
}

#[tokio::test]
async fn the_callback_trades_the_code_with_the_application_credentials() {
    let app = TestApp::new().await;
    app.sso
        .sign_in_as(95_000_001, "Test Pilot", "owner-hash", &[]);
    esi_knows(&app, 95_000_001, 98_000_001);

    app.eve_login(&Browser::guest(), "").await;

    let token = &app.sso.requests()[0];
    assert_eq!(token.path, "/v2/oauth/token");
    assert_eq!(
        token.header("authorization"),
        Some("Basic dGVzdC1jbGllbnQtaWQ6dGVzdC1jbGllbnQtc2VjcmV0")
    );
    assert_eq!(
        token.text(),
        "grant_type=authorization_code&code=valid-code"
    );
}

#[tokio::test]
async fn logging_out_ends_the_session() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    let browser = app.sign_in(user).await;

    let response = app.post(&browser, "/logout", json!({})).await;

    assert_eq!(response.assert_done(), Some("/".to_owned()));
    assert_eq!(app.count("sessions").await, 0);
    app.page(&browser, "/dashboard").await.assert_status(302);
}

//! Every form in the world (common::world) as the pages send it: where it
//! sends the browser, the flash message on the next page, the validation
//! messages, the refusals and what it leaves in the database.

use axum::http::Method;
use serde_json::{Value, json};

use crate::common::world::{ADMIN, ADMIN_ALT, EMILE, FRESH, MEMBER, NICOLAS, RED, TINA};
use crate::common::{Browser, TestApp, TestResponse, assert_json_eq};

/// The flash message the page the form sent the browser to shows.
async fn flash(app: &TestApp, browser: &Browser, response: &TestResponse, page: &str) -> Value {
    let redirect = response.assert_done();
    let next = response.browser(browser);
    let target = redirect.unwrap_or_else(|| page.to_owned());
    let props = app.page(&next, &target).await;
    props.assert_status(200);
    props.json()["flash"].clone()
}

fn success(message: &str) -> Value {
    json!({ "success": message, "error": null, "info": null })
}

fn error(message: &str) -> Value {
    json!({ "success": null, "error": message, "info": null })
}

fn info(message: &str) -> Value {
    json!({ "success": null, "error": null, "info": message })
}

async fn discord_settings(app: &TestApp) -> Vec<Value> {
    app.rows("select webhook_url, role_id from discord_settings order by id")
        .await
}

async fn request_statuses(app: &TestApp) -> Vec<Value> {
    app.rows("select id, status from standing_requests order by id")
        .await
}

// Discord settings

#[tokio::test]
async fn the_admin_saves_the_discord_settings() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(1, ADMIN).await;

    let response = app
        .put(
            &browser,
            "/admin/discord-settings",
            json!({ "webhook_url": "https://discord.com/api/webhooks/1/abc", "role_id": "123456" }),
        )
        .await;

    assert_json_eq(
        &success("Discord settings updated."),
        &flash(&app, &browser, &response, "/admin/settings").await,
    );
    assert_json_eq(
        &json!([{ "webhook_url": "https://discord.com/api/webhooks/1/abc", "role_id": "123456" }]),
        &json!(discord_settings(&app).await),
    );
}

#[tokio::test]
async fn the_admin_clears_the_discord_settings() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(1, ADMIN).await;

    let response = app
        .put(
            &browser,
            "/admin/discord-settings",
            json!({ "webhook_url": "", "role_id": null }),
        )
        .await;

    assert_json_eq(
        &success("Discord settings updated."),
        &flash(&app, &browser, &response, "/admin/settings").await,
    );
    assert_json_eq(
        &json!([{ "webhook_url": null, "role_id": null }]),
        &json!(discord_settings(&app).await),
    );
}

#[tokio::test]
async fn the_first_discord_settings_are_created() {
    let app = TestApp::world().await;
    app.exec("delete from discord_settings").await;
    let browser = app.sign_in_as(1, ADMIN).await;

    let response = app
        .put(
            &browser,
            "/admin/discord-settings",
            json!({ "webhook_url": "https://discord.com/api/webhooks/2/x" }),
        )
        .await;

    assert_json_eq(
        &success("Discord settings updated."),
        &flash(&app, &browser, &response, "/admin/settings").await,
    );
    assert_json_eq(
        &json!([{ "webhook_url": "https://discord.com/api/webhooks/2/x", "role_id": null }]),
        &json!(discord_settings(&app).await),
    );
}

#[tokio::test]
async fn the_discord_settings_are_validated() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(1, ADMIN).await;
    let before = discord_settings(&app).await;

    for (body, errors) in [
        (
            json!({ "webhook_url": "not-a-url", "role_id": "abc" }),
            json!({
                "webhook_url": "The webhook url field must be a valid URL.",
                "role_id": "The role id field format is invalid."
            }),
        ),
        (
            json!({ "webhook_url": "http://discord.com/api/webhooks/1/abc", "role_id": "12 3" }),
            json!({
                "webhook_url": "The webhook url field must start with one of the following: https://.",
                "role_id": "The role id field format is invalid."
            }),
        ),
        (
            json!({ "webhook_url": null, "role_id": 123 }),
            json!({ "role_id": "The role id field must be a string." }),
        ),
    ] {
        let response = app.put(&browser, "/admin/discord-settings", body).await;
        assert_json_eq(&errors, &response.assert_invalid());
    }

    assert_eq!(discord_settings(&app).await, before);
}

#[tokio::test]
async fn a_pilot_cannot_use_the_admin_forms() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    for (method, path, body) in [
        (
            Method::PUT,
            "/admin/discord-settings",
            json!({ "webhook_url": "https://discord.com/x" }),
        ),
        (
            Method::PUT,
            "/admin/standing-requests/1",
            json!({ "status": "done" }),
        ),
        (
            Method::PUT,
            "/admin/standings",
            json!({ "type": "corporation" }),
        ),
    ] {
        let response = app.form(&browser, method, path, body).await;
        response.assert_status(403);
        assert_eq!(
            response.json()["message"],
            "This action is unauthorized.",
            "{path}"
        );
    }

    assert_eq!(app.request_status(1).await, "pending");
    assert_eq!(
        app.rows("select type, entity_id from standings_sources")
            .await,
        [json!({ "type": "alliance", "entity_id": 99000002 })]
    );
}

// Standing requests in the admin console

#[tokio::test]
async fn the_admin_resolves_requests() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(1, ADMIN).await;

    for (id, status) in [(1, "done"), (2, "rejected")] {
        let response = app
            .put(
                &browser,
                &format!("/admin/standing-requests/{id}"),
                json!({ "status": status }),
            )
            .await;
        assert_json_eq(
            &success("Standing request updated."),
            &flash(&app, &browser, &response, "/admin/standing-requests").await,
        );
    }

    assert_json_eq(
        &json!([
            { "id": 1, "status": "done" },
            { "id": 2, "status": "rejected" },
            { "id": 3, "status": "rejected" },
            { "id": 4, "status": "done" },
            { "id": 5, "status": "pending" }
        ]),
        &json!(request_statuses(&app).await),
    );
}

#[tokio::test]
async fn a_request_resolves_only_to_done_or_rejected() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(1, ADMIN).await;

    let response = app
        .put(
            &browser,
            "/admin/standing-requests/1",
            json!({ "status": "maybe" }),
        )
        .await;

    assert_json_eq(
        &json!({ "status": "The selected status is invalid." }),
        &response.assert_invalid(),
    );
    assert_eq!(app.request_status(1).await, "pending");
}

#[tokio::test]
async fn resolving_an_unknown_request_is_not_found() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(1, ADMIN).await;

    let response = app
        .put(
            &browser,
            "/admin/standing-requests/999",
            json!({ "status": "done" }),
        )
        .await;

    response.assert_status(404);
    assert_eq!(
        response.json()["message"],
        "No query results for model [App\\Models\\StandingRequest] 999"
    );
}

// The standings source

async fn source(app: &TestApp) -> Vec<Value> {
    app.rows("select type, entity_id, last_synced_at is not null as synced from standings_sources")
        .await
}

#[tokio::test]
async fn the_admin_picks_the_source_from_the_admin_character() {
    for (kind, entity_id) in [
        ("character", 2_112_000_001_i64),
        ("corporation", 98_000_001),
        ("alliance", 99_000_002),
    ] {
        let app = TestApp::world().await;
        let browser = app.sign_in_as(1, ADMIN).await;

        let response = app
            .put(&browser, "/admin/standings", json!({ "type": kind }))
            .await;

        assert_json_eq(
            &success("Standings source updated."),
            &flash(&app, &browser, &response, "/admin/settings").await,
        );
        assert_json_eq(
            &json!([{ "type": kind, "entity_id": entity_id, "synced": false }]),
            &json!(source(&app).await),
        );
        assert!(app.jobs().await.is_empty());
    }
}

#[tokio::test]
async fn the_source_is_picked_from_the_admin_character_while_an_alt_is_active() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(1, ADMIN_ALT).await;

    let response = app
        .put(&browser, "/admin/standings", json!({ "type": "alliance" }))
        .await;

    assert_json_eq(
        &success("Standings source updated."),
        &flash(&app, &browser, &response, "/admin/settings").await,
    );
    assert_json_eq(
        &json!([{ "type": "alliance", "entity_id": 99000002, "synced": false }]),
        &json!(source(&app).await),
    );
}

#[tokio::test]
async fn the_source_type_is_validated() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(1, ADMIN).await;

    for (body, errors) in [
        (
            json!({ "type": "faction" }),
            json!({ "type": "The selected type is invalid." }),
        ),
        (json!({}), json!({ "type": "The type field is required." })),
    ] {
        let response = app.put(&browser, "/admin/standings", body).await;
        assert_json_eq(&errors, &response.assert_invalid());
    }

    assert_json_eq(
        &json!([{ "type": "alliance", "entity_id": 99000002, "synced": true }]),
        &json!(source(&app).await),
    );
}

/// The admin's sync reads the source at once: new names are looked up,
/// a request whose subject turned blue is done, and every opted-in
/// character that does not inherit the source gets a sync.
#[tokio::test]
async fn the_admin_syncs_the_standings_now() {
    let app = TestApp::world().await;
    app.esi.contacts(
        "alliances/99000002",
        &[
            (99_000_001, "alliance", 10.0),
            (98_000_002, "corporation", 10.0),
            (95_000_100, "character", 5.0),
            (95_000_200, "character", 10.0),
            (98_000_003, "corporation", -5.0),
            (500_001, "faction", -5.0),
            (99_000_003, "alliance", -10.0),
            (95_000_101, "character", -10.0),
            (98_000_004, "corporation", -5.0),
            (95_000_010, "character", 7.5),
        ],
    );
    app.esi.name(95_000_200, "character", "New Friend");
    app.esi
        .affiliation(95_000_200, 98_000_002, Some(99_000_001));
    let browser = app.sign_in_as(1, ADMIN).await;

    let response = app.post(&browser, "/admin/standings/sync", json!({})).await;

    assert_json_eq(
        &success("Standings sync started."),
        &flash(&app, &browser, &response, "/admin/settings").await,
    );
    assert_json_eq(
        &json!([
            { "id": 1, "status": "done" },
            { "id": 2, "status": "pending" },
            { "id": 3, "status": "rejected" },
            { "id": 4, "status": "done" },
            { "id": 5, "status": "pending" }
        ]),
        &json!(request_statuses(&app).await),
    );
    assert_json_eq(
        &json!([
            { "character_id": 95000001 },
            { "character_id": 95000020 },
            { "character_id": 2112000002 }
        ]),
        &json!(app.jobs_of("character_sync").await),
    );
    assert_json_eq(
        &json!([
            { "contact_id": 500001, "standing": -5 },
            { "contact_id": 95000010, "standing": 7.5 },
            { "contact_id": 95000100, "standing": 5 },
            { "contact_id": 95000101, "standing": -10 },
            { "contact_id": 95000200, "standing": 10 },
            { "contact_id": 98000002, "standing": 10 },
            { "contact_id": 98000003, "standing": -5 },
            { "contact_id": 98000004, "standing": -5 },
            { "contact_id": 99000001, "standing": 10 },
            { "contact_id": 99000003, "standing": -10 }
        ]),
        &json!(
            app.rows(
                "select contact_id, standing::float8 as standing from source_contacts
                 order by contact_id"
            )
            .await
        ),
    );
    assert_eq!(
        app.scalar("select name from characters where id = 95000200")
            .await,
        "New Friend"
    );
}

// Signing out

#[tokio::test]
async fn a_pilot_signs_out() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    let response = app.post(&browser, "/logout", json!({})).await;

    assert_eq!(response.assert_done().as_deref(), Some("/"));
    let next = response.browser(&browser);
    assert!(next.cookie.is_none());
    app.page(&next, "/dashboard").await.assert_status(302);
}

// Characters

#[tokio::test]
async fn removing_the_active_main_switches_to_another_character() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    let response = app
        .delete(&browser, &format!("/auth/character/{NICOLAS}"))
        .await;

    assert_eq!(response.assert_done(), None);
    let next = response.browser(&browser);
    let props = app.page(&next, "/dashboard").await.json();
    assert_json_eq(
        &success("Nicolas Pilot has been removed from your account."),
        &props["flash"],
    );
    assert_eq!(props["auth"]["active_character"]["id"], TINA);
    assert_eq!(app.main_character_of(2).await, None);
    assert_eq!(app.owner_of(NICOLAS).await, None);
    assert!(!app.should_sync(NICOLAS).await);
}

#[tokio::test]
async fn removing_an_alt_keeps_the_main() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    let response = app
        .delete(&browser, &format!("/auth/character/{EMILE}"))
        .await;

    assert_json_eq(
        &success("Émile Pilot has been removed from your account."),
        &flash(&app, &browser, &response, "/dashboard").await,
    );
    assert_eq!(app.main_character_of(2).await, Some(NICOLAS));
    assert_eq!(app.owner_of(EMILE).await, None);
}

#[tokio::test]
async fn removing_the_last_character_deletes_the_account() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(3, FRESH).await;

    let response = app
        .delete(&browser, &format!("/auth/character/{FRESH}"))
        .await;

    assert_eq!(response.assert_done().as_deref(), Some("/"));
    let next = response.browser(&browser);
    assert_json_eq(
        &success("Your account has been deleted."),
        &app.page(&next, "/").await.json()["flash"],
    );
    assert!(!app.user_exists(3).await);
}

#[tokio::test]
async fn a_pilot_cannot_touch_another_pilots_character() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    for (method, path, body) in [
        (Method::DELETE, format!("/auth/character/{RED}"), json!({})),
        (Method::PUT, format!("/auth/character/{RED}"), json!({})),
        (
            Method::PUT,
            format!("/auth/character/{RED}/main"),
            json!({}),
        ),
        (
            Method::PUT,
            format!("/standings/{RED}"),
            json!({ "should_sync": true }),
        ),
        (
            Method::POST,
            format!("/standings/{RED}/request"),
            json!({ "type": "character" }),
        ),
    ] {
        let response = app.form(&browser, method, &path, body).await;
        response.assert_status(403);
        assert_eq!(
            response.json()["message"],
            "This action is unauthorized.",
            "{path}"
        );
    }

    assert_eq!(app.owner_of(RED).await, Some(4));
    assert_eq!(app.main_character_of(2).await, Some(NICOLAS));
    assert!(app.jobs().await.is_empty());
}

#[tokio::test]
async fn a_pilot_switches_the_active_character() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    let response = app
        .put(&browser, &format!("/auth/character/{TINA}"), json!({}))
        .await;

    assert_eq!(response.assert_done(), None);
    let next = response.browser(&browser);
    let props = app.page(&next, "/dashboard").await.json();
    assert_json_eq(&success("Switched to Tina Pilot."), &props["flash"]);
    assert_eq!(props["auth"]["active_character"]["id"], TINA);
}

#[tokio::test]
async fn a_pilot_picks_a_main() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(4, RED).await;

    let response = app
        .put(&browser, &format!("/auth/character/{RED}/main"), json!({}))
        .await;

    assert_json_eq(
        &success("Red Pilot is now your main character."),
        &flash(&app, &browser, &response, "/dashboard").await,
    );
    assert_eq!(app.main_character_of(4).await, Some(RED));
}

#[tokio::test]
async fn picking_a_main_completes_the_setup() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(3, FRESH).await;

    let response = app
        .put(
            &browser,
            &format!("/auth/character/{FRESH}/main"),
            json!({}),
        )
        .await;

    assert_json_eq(
        &success("Fresh Pilot is now your main character."),
        &flash(&app, &browser, &response, "/dashboard").await,
    );
    assert_eq!(app.main_character_of(3).await, Some(FRESH));
    assert_eq!(
        app.scalar("select setup_completed_at is not null from users where id = 3")
            .await,
        true
    );
}

#[tokio::test]
async fn skipping_the_setup_completes_it_without_a_main() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(3, FRESH).await;

    let response = app.post(&browser, "/setup/complete", json!({})).await;

    assert_json_eq(
        &json!({ "success": null, "error": null, "info": null }),
        &flash(&app, &browser, &response, "/dashboard").await,
    );
    assert_eq!(app.main_character_of(3).await, None);
    assert_eq!(
        app.scalar("select setup_completed_at is not null from users where id = 3")
            .await,
        true
    );
}

// Syncing

#[tokio::test]
async fn a_pilot_switches_syncing_on_and_off() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    for (character, body, message, synced) in [
        (
            EMILE,
            json!({ "should_sync": true }),
            "Émile Pilot will now sync standings.",
            true,
        ),
        (
            NICOLAS,
            json!({ "should_sync": false }),
            "Nicolas Pilot will no longer sync standings.",
            false,
        ),
        (
            EMILE,
            json!({ "should_sync": "0" }),
            "Émile Pilot will no longer sync standings.",
            false,
        ),
        (
            EMILE,
            json!({ "should_sync": "1" }),
            "Émile Pilot will now sync standings.",
            true,
        ),
    ] {
        let response = app
            .put(&browser, &format!("/standings/{character}"), body)
            .await;
        assert_json_eq(
            &success(message),
            &flash(&app, &browser, &response, "/dashboard").await,
        );
        assert_eq!(app.should_sync(character).await, synced, "{message}");
    }
}

#[tokio::test]
async fn the_sync_switch_is_validated() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    for (body, message) in [
        (
            json!({ "should_sync": "maybe" }),
            "The should sync field must be true or false.",
        ),
        (json!({}), "The should sync field is required."),
    ] {
        let response = app
            .put(&browser, &format!("/standings/{EMILE}"), body)
            .await;
        assert_json_eq(
            &json!({ "should_sync": message }),
            &response.assert_invalid(),
        );
    }

    assert!(!app.should_sync(EMILE).await);
}

#[tokio::test]
async fn switching_an_unknown_character_is_not_found() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    let response = app
        .put(&browser, "/standings/1", json!({ "should_sync": true }))
        .await;

    response.assert_status(404);
    assert_eq!(
        response.json()["message"],
        "No query results for model [App\\Models\\Character] 1"
    );
}

/// Only opted-in characters with the write scope that do not inherit
/// the source are synced.
#[tokio::test]
async fn a_pilot_syncs_now() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    let response = app.post(&browser, "/standings/sync", json!({})).await;

    assert_json_eq(
        &success("Re-syncing 1 character(s)."),
        &flash(&app, &browser, &response, "/dashboard").await,
    );
    assert_json_eq(
        &json!([{ "character_id": 95000001 }]),
        &json!(app.jobs_of("character_sync").await),
    );
}

#[tokio::test]
async fn there_is_nothing_to_sync_now_without_an_eligible_character() {
    let app = TestApp::world().await;

    for (user, character) in [(3, FRESH), (5, MEMBER)] {
        let browser = app.sign_in_as(user, character).await;
        let response = app.post(&browser, "/standings/sync", json!({})).await;
        assert_json_eq(
            &info("No characters are set to sync."),
            &flash(&app, &browser, &response, "/dashboard").await,
        );
    }

    assert!(app.jobs().await.is_empty());
}

// Requesting standings

#[tokio::test]
async fn a_pilot_requests_standings() {
    for (kind, setup, subject) in [
        ("character", None, RED),
        (
            "corporation",
            Some("delete from source_contacts where contact_id = 98000004"),
            98_000_004,
        ),
        (
            "alliance",
            Some("delete from source_contacts where contact_id = 99000003"),
            99_000_003,
        ),
    ] {
        let app = TestApp::world().await;
        if let Some(setup) = setup {
            app.exec(setup).await;
        }
        let browser = app.sign_in_as(4, RED).await;

        let response = app
            .post(
                &browser,
                &format!("/standings/{RED}/request"),
                json!({ "type": kind }),
            )
            .await;

        assert_json_eq(
            &success("Standing request submitted."),
            &flash(&app, &browser, &response, "/dashboard").await,
        );
        let request = app
            .rows(&format!(
                "select id, subject_type, subject_id, requested_by_character_id, status
                 from standing_requests where subject_id = {subject}"
            ))
            .await;
        // A rejected request is asked again; anything else is new.
        let id = if kind == "alliance" { 3 } else { 6 };
        assert_json_eq(
            &json!([{
                "id": id,
                "subject_type": kind,
                "subject_id": subject,
                "requested_by_character_id": RED,
                "status": "pending"
            }]),
            &json!(request),
        );
        assert_json_eq(
            &json!([{ "standing_request_id": id }]),
            &json!(app.jobs_of("discord_request").await),
        );
    }
}

#[tokio::test]
async fn a_requested_standing_is_not_asked_again() {
    let app = TestApp::world().await;

    for (user, character) in [(3, FRESH), (2, NICOLAS)] {
        let browser = app.sign_in_as(user, character).await;
        let response = app
            .post(
                &browser,
                &format!("/standings/{character}/request"),
                json!({ "type": "character" }),
            )
            .await;
        assert_json_eq(
            &info("This standing has already been requested."),
            &flash(&app, &browser, &response, "/dashboard").await,
        );
    }

    assert_eq!(app.count("standing_requests").await, 5);
    assert!(app.jobs().await.is_empty());
}

/// A subject with a standing, a missing alliance, the source itself or a
/// missing source cannot be requested.
#[tokio::test]
async fn some_standings_cannot_be_requested() {
    for (setup, user, character, path, kind) in [
        (
            None,
            2,
            NICOLAS,
            format!("/standings/{TINA}/request"),
            "character",
        ),
        (
            None,
            3,
            FRESH,
            format!("/standings/{FRESH}/request"),
            "alliance",
        ),
        (
            None,
            5,
            MEMBER,
            format!("/standings/{MEMBER}/request"),
            "alliance",
        ),
        (
            Some("delete from standings_sources"),
            4,
            RED,
            format!("/standings/{RED}/request"),
            "character",
        ),
    ] {
        let app = TestApp::world().await;
        if let Some(setup) = setup {
            app.exec(setup).await;
        }
        let browser = app.sign_in_as(user, character).await;

        let response = app.post(&browser, &path, json!({ "type": kind })).await;

        assert_json_eq(
            &error("This standing cannot be requested."),
            &flash(&app, &browser, &response, "/dashboard").await,
        );
        assert_eq!(app.count("standing_requests").await, 5, "{path}");
        assert!(app.jobs().await.is_empty());
    }
}

#[tokio::test]
async fn the_request_type_is_validated() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(4, RED).await;

    let response = app
        .post(
            &browser,
            &format!("/standings/{RED}/request"),
            json!({ "type": "faction" }),
        )
        .await;

    assert_json_eq(
        &json!({ "type": "The selected type is invalid." }),
        &response.assert_invalid(),
    );
    assert_eq!(app.count("standing_requests").await, 5);
}

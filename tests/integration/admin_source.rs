//! tests/Feature/Admin/StandingsSourceTest.php: picking the standings
//! source and the admin's "Sync now".

use serde_json::json;

use crate::common::factories::{NewCharacter, Owner};
use crate::common::{
    Browser, READ_CORPORATION_CONTACTS, TestApp, WRITE_CHARACTER_CONTACTS, assert_json_eq,
};

/// `adminUser()`: the admin character in corporation 2000 and alliance
/// 3000.
async fn admin(app: &mut TestApp) -> (i64, i64, Browser) {
    let user = app.user().await;
    app.corporation(2000, Some("Admin Corp"), Some("ADMC"), None)
        .await;
    app.alliance(3000, Some("Admin Alliance"), Some("ADMA"))
        .await;
    let character = app
        .character(NewCharacter {
            owner: Owner::User(user),
            corporation_id: Some(2000),
            alliance_id: Some(3000),
            ..NewCharacter::default()
        })
        .await;
    app.admins(&[character]);
    let browser = app.sign_in(user).await;
    (user, character, browser)
}

async fn source(app: &TestApp) -> serde_json::Value {
    json!(
        app.rows("select type, entity_id from standings_sources")
            .await
    )
}

#[tokio::test]
async fn forbids_non_admins_from_updating_the_source() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    app.admins(&[999_999]);

    app.put(
        &app.sign_in(user).await,
        "/admin/standings",
        json!({ "type": "corporation" }),
    )
    .await
    .assert_status(403);
    assert_json_eq(&json!([]), &source(&app).await);
}

#[tokio::test]
async fn derives_the_entity_id_from_the_admin_character() {
    let mut app = TestApp::new().await;
    let (_, _, browser) = admin(&mut app).await;

    app.put(
        &browser,
        "/admin/standings",
        json!({ "type": "corporation" }),
    )
    .await
    .assert_done();

    assert_json_eq(
        &json!([{ "type": "corporation", "entity_id": 2000 }]),
        &source(&app).await,
    );
}

#[tokio::test]
async fn uses_the_alliance_id_when_the_alliance_is_selected() {
    let mut app = TestApp::new().await;
    let (_, _, browser) = admin(&mut app).await;

    app.put(&browser, "/admin/standings", json!({ "type": "alliance" }))
        .await
        .assert_done();

    assert_json_eq(
        &json!([{ "type": "alliance", "entity_id": 3000 }]),
        &source(&app).await,
    );
}

#[tokio::test]
async fn rejects_a_source_type_the_admin_character_does_not_have() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.admins(&[character]);

    let errors = app
        .put(
            &app.sign_in(user).await,
            "/admin/standings",
            json!({ "type": "alliance" }),
        )
        .await
        .assert_invalid();

    assert_json_eq(
        &json!({ "type": "Your character has no alliance to use as a source." }),
        &errors,
    );
    assert_json_eq(&json!([]), &source(&app).await);
}

#[tokio::test]
async fn clears_canonical_contacts_when_the_source_changes() {
    let mut app = TestApp::new().await;
    let (_, _, browser) = admin(&mut app).await;
    app.source("character", 1).await;
    for _ in 0..3 {
        app.any_source_contact().await;
    }

    app.put(
        &browser,
        "/admin/standings",
        json!({ "type": "corporation" }),
    )
    .await
    .assert_done();

    assert_eq!(app.count("source_contacts").await, 0);
    assert_json_eq(
        &json!([{ "type": "corporation", "entity_id": 2000 }]),
        &source(&app).await,
    );
}

#[tokio::test]
async fn validates_the_source_type() {
    let mut app = TestApp::new().await;
    let (_, _, browser) = admin(&mut app).await;

    let errors = app
        .put(&browser, "/admin/standings", json!({ "type": "invalid" }))
        .await
        .assert_invalid();

    assert_json_eq(&json!({ "type": "The selected type is invalid." }), &errors);
}

#[tokio::test]
async fn queues_a_sync_for_opted_in_characters_when_the_admin_clicks_sync_now() {
    let mut app = TestApp::new().await;
    let (_, admin_character, browser) = admin(&mut app).await;
    app.grant_scopes(admin_character, &[READ_CORPORATION_CONTACTS])
        .await;
    app.source("corporation", 2000).await;
    let syncable = app
        .character(NewCharacter {
            should_sync: true,
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(syncable, &[WRITE_CHARACTER_CONTACTS])
        .await;
    app.esi
        .contacts("corporations/2000", &[(100, "character", 5.0)]);
    app.esi.name(100, "character", "Contact One");
    app.esi.affiliation(100, 2001, None);
    app.esi.corporation(2001, "Contact Corp", None);

    app.post(&browser, "/admin/standings/sync", json!({}))
        .await
        .assert_done();

    assert_json_eq(
        &json!([{ "character_id": syncable }]),
        &json!(app.jobs_of("character_sync").await),
    );
}

/// FIXES.md, ""Sync now" said the sync started when the source could not
/// be read".
#[tokio::test]
async fn sync_now_reports_an_unreadable_source() {
    let mut app = TestApp::new().await;
    let (_, _, browser) = admin(&mut app).await;
    // The admin holds no token that can read the source.
    app.source("corporation", 2000).await;

    let redirect = app
        .post(&browser, "/admin/standings/sync", json!({}))
        .await
        .assert_done();
    assert_eq!(redirect, None);

    let props = app.page(&browser, "/admin/settings").await.json();
    assert_json_eq(
        &json!({
            "success": null,
            "error": "The standings source could not be read.",
            "info": null,
        }),
        &props["flash"],
    );
}

//! tests/Feature/Standings/CharacterStandingsSyncTest.php: writing the
//! source's standings to one character's in-game contacts.

use crate::common::factories::NewCharacter;
use crate::common::{READ_CHARACTER_CONTACTS, TestApp, WRITE_CHARACTER_CONTACTS, assert_json_eq};

/// `syncingCharacter()`: opted in, with both contact scopes.
async fn syncing_character(app: &TestApp) -> i64 {
    let character = app
        .character(NewCharacter {
            should_sync: true,
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(
        character,
        &[READ_CHARACTER_CONTACTS, WRITE_CHARACTER_CONTACTS],
    )
    .await;
    character
}

/// `fakeContacts()`: the character's current contacts.
fn current_contacts(app: &TestApp, character: i64, contacts: &[(i64, f64)]) {
    let contacts: Vec<(i64, &str, f64)> = contacts
        .iter()
        .map(|(id, standing)| (*id, "character", *standing))
        .collect();
    app.esi
        .contacts(&format!("characters/{character}"), &contacts);
}

async fn sync(app: &TestApp, character: i64) {
    bluebook::standings::character::sync(&app.state, character)
        .await
        .expect("a sync");
}

fn writes(app: &TestApp, method: &str) -> Vec<crate::common::mocks::Recorded> {
    app.esi
        .requests()
        .into_iter()
        .filter(|request| request.method.as_str() == method)
        .collect()
}

#[tokio::test]
async fn adds_new_source_contacts_to_a_syncing_character() {
    let app = TestApp::new().await;
    let character = syncing_character(&app).await;
    app.source_contact(100, "character", 5.0).await;
    app.source_contact(200, "character", -10.0).await;
    current_contacts(&app, character, &[]);

    sync(&app, character).await;

    let posts = writes(&app, "POST");
    assert!(posts.iter().any(|post| {
        post.path == format!("/characters/{character}/contacts/")
            && post
                .json()
                .as_array()
                .is_some_and(|ids| ids.contains(&100.into()))
    }));
    assert_json_eq(
        &serde_json::json!([
            { "contact_id": 100, "standing": 5 },
            { "contact_id": 200, "standing": -10 },
        ]),
        &serde_json::json!(
            app.rows(&format!(
                "select contact_id, standing::float8 as standing from character_synced_contacts
             where character_id = {character} order by contact_id"
            ))
            .await
        ),
    );
}

#[tokio::test]
async fn updates_contacts_whose_standing_changed() {
    let app = TestApp::new().await;
    let character = syncing_character(&app).await;
    app.source_contact(100, "character", 10.0).await;
    current_contacts(&app, character, &[(100, 5.0)]);

    sync(&app, character).await;

    let puts = writes(&app, "PUT");
    assert_eq!(puts.len(), 1);
    assert_eq!(puts[0].path, format!("/characters/{character}/contacts/"));
    assert_eq!(puts[0].json(), serde_json::json!([100]));
}

#[tokio::test]
async fn does_not_write_contacts_that_already_match_the_source() {
    let app = TestApp::new().await;
    let character = syncing_character(&app).await;
    app.source_contact(100, "character", 5.0).await;
    current_contacts(&app, character, &[(100, 5.0)]);

    sync(&app, character).await;

    assert_eq!(writes(&app, "POST").len(), 0);
    assert_eq!(writes(&app, "PUT").len(), 0);
}

#[tokio::test]
async fn removes_a_previously_synced_contact_once_the_source_drops_it() {
    let app = TestApp::new().await;
    let character = syncing_character(&app).await;
    app.source_contact(100, "character", 5.0).await;
    app.synced_contact(character, 100, 5.0).await;
    app.synced_contact(character, 999, -10.0).await;
    // The character still has 100 and 999, plus a personal contact 555.
    current_contacts(&app, character, &[(100, 5.0), (999, -10.0), (555, 10.0)]);

    sync(&app, character).await;

    let deletes = writes(&app, "DELETE");
    assert_eq!(deletes.len(), 1);
    assert_eq!(deletes[0].query.as_deref(), Some("contact_ids=999"));
    assert_json_eq(
        &serde_json::json!([{ "contact_id": 100 }]),
        &serde_json::json!(
            app.rows(&format!(
                "select contact_id from character_synced_contacts where character_id = {character}"
            ))
            .await
        ),
    );
}

#[tokio::test]
async fn never_deletes_personal_contacts_that_were_never_synced() {
    let app = TestApp::new().await;
    let character = syncing_character(&app).await;
    app.source_contact(100, "character", 5.0).await;
    // 555 is a personal contact, not in the source nor the ledger.
    current_contacts(&app, character, &[(100, 5.0), (555, 10.0)]);

    sync(&app, character).await;

    assert_eq!(writes(&app, "DELETE").len(), 0);
}

#[tokio::test]
async fn skips_characters_that_have_not_opted_in() {
    let app = TestApp::new().await;
    let character = app.character(NewCharacter::default()).await;
    app.grant_scopes(
        character,
        &[READ_CHARACTER_CONTACTS, WRITE_CHARACTER_CONTACTS],
    )
    .await;
    app.source_contact(100, "character", 5.0).await;

    sync(&app, character).await;

    assert_eq!(app.esi.requests().len(), 0);
}

#[tokio::test]
async fn skips_characters_without_the_write_scope() {
    let app = TestApp::new().await;
    let character = app
        .character(NewCharacter {
            should_sync: true,
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(character, &[READ_CHARACTER_CONTACTS])
        .await;
    app.source_contact(100, "character", 5.0).await;

    sync(&app, character).await;

    assert_eq!(app.esi.requests().len(), 0);
}

#[tokio::test]
async fn skips_characters_that_already_inherit_the_source_corporation_standings() {
    let app = TestApp::new().await;
    app.corporation(2000, Some("Source Corp"), None, None).await;
    let character = app
        .character(NewCharacter {
            should_sync: true,
            corporation_id: Some(2000),
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(
        character,
        &[READ_CHARACTER_CONTACTS, WRITE_CHARACTER_CONTACTS],
    )
    .await;
    app.source("corporation", 2000).await;
    app.source_contact(100, "character", 5.0).await;

    sync(&app, character).await;

    assert_eq!(app.esi.requests().len(), 0);
}

// New: the chunks ESI allows (100 ids per add or edit, 20 per delete).

#[tokio::test]
async fn writes_in_chunks_of_one_hundred_and_deletes_in_chunks_of_twenty() {
    let app = TestApp::new().await;
    let character = syncing_character(&app).await;
    for id in 1..=150 {
        app.source_contact(10_000 + id, "character", 5.0).await;
    }
    let stale: Vec<(i64, f64)> = (1..=25).map(|id| (20_000 + id, 5.0)).collect();
    for (id, standing) in &stale {
        app.synced_contact(character, *id, *standing).await;
    }
    current_contacts(&app, character, &stale);

    sync(&app, character).await;

    let sizes = |method: &str| -> Vec<usize> {
        writes(&app, method)
            .iter()
            .map(|request| match method {
                "DELETE" => request
                    .query_param("contact_ids")
                    .expect("ids")
                    .split(',')
                    .count(),
                _ => request.json().as_array().map_or(0, Vec::len),
            })
            .collect()
    };
    assert_eq!(sizes("POST"), vec![100, 50]);
    assert_eq!(sizes("DELETE"), vec![20, 5]);
    let post = &writes(&app, "POST")[0];
    assert_eq!(post.query.as_deref(), Some("standing=5&watched=0"));
}

async fn ledger(app: &TestApp, character: i64) -> serde_json::Value {
    serde_json::json!(
        app.rows(&format!(
            "select contact_id, standing::float8 as standing from character_synced_contacts
             where character_id = {character} order by contact_id"
        ))
        .await
    )
}

/// FIXES.md, "A failed read of a character's contacts counted as no
/// contacts".
#[tokio::test]
async fn a_failed_read_of_the_contacts_aborts_the_sync() {
    let app = TestApp::new().await;
    let character = syncing_character(&app).await;
    app.source_contact(100, "character", 5.0).await;
    app.synced_contact(character, 300, 5.0).await;
    app.esi
        .fail_path(&format!("GET /characters/{character}/contacts/"), 503);

    sync(&app, character).await;

    assert!(
        app.esi
            .requests()
            .iter()
            .all(|request| request.method.as_str() == "GET")
    );
    assert_json_eq(
        &serde_json::json!([{ "contact_id": 300, "standing": 5 }]),
        &ledger(&app, character).await,
    );
}

/// FIXES.md, "The ledger recorded writes ESI refused".
#[tokio::test]
async fn the_ledger_records_only_writes_that_succeeded() {
    let app = TestApp::new().await;
    let character = syncing_character(&app).await;
    app.source_contact(100, "character", 5.0).await;
    app.source_contact(200, "character", 10.0).await;
    app.source_contact(300, "character", -5.0).await;
    app.source_contact(400, "character", 5.0).await;
    app.synced_contact(character, 300, -10.0).await;
    app.synced_contact(character, 400, 1.0).await;
    current_contacts(&app, character, &[(200, 10.0), (300, -10.0), (400, 1.0)]);
    // The add of 100 (+5) fails, the edit to -5 succeeds, the edit to +5
    // fails.
    app.esi.queue(
        &format!("POST /characters/{character}/contacts/"),
        vec![crate::common::Canned {
            status: 520,
            body: serde_json::json!({ "error": "unknown" }),
        }],
    );
    app.esi.queue(
        &format!("PUT /characters/{character}/contacts/"),
        vec![
            crate::common::Canned {
                status: 200,
                body: serde_json::json!({}),
            },
            crate::common::Canned {
                status: 503,
                body: serde_json::json!({ "error": "unavailable" }),
            },
        ],
    );

    sync(&app, character).await;

    assert_eq!(writes(&app, "POST").len(), 1);
    assert_eq!(writes(&app, "PUT").len(), 2);
    assert_json_eq(
        &serde_json::json!([
            { "contact_id": 200, "standing": 10 },
            { "contact_id": 300, "standing": -5 },
            { "contact_id": 400, "standing": 1 },
        ]),
        &ledger(&app, character).await,
    );
}

#[tokio::test]
async fn a_failed_delete_keeps_its_ledger_row() {
    let app = TestApp::new().await;
    let character = syncing_character(&app).await;
    app.synced_contact(character, 500, 5.0).await;
    app.synced_contact(character, 600, 5.0).await;
    // 600 is gone from the game already: nothing to delete, the row goes.
    current_contacts(&app, character, &[(500, 5.0)]);
    app.esi
        .fail_path(&format!("DELETE /characters/{character}/contacts/"), 502);

    sync(&app, character).await;

    assert_eq!(writes(&app, "DELETE").len(), 1);
    assert_json_eq(
        &serde_json::json!([{ "contact_id": 500, "standing": 5 }]),
        &ledger(&app, character).await,
    );
}

#[tokio::test]
async fn a_successful_delete_removes_its_ledger_row() {
    let app = TestApp::new().await;
    let character = syncing_character(&app).await;
    app.synced_contact(character, 500, 5.0).await;
    current_contacts(&app, character, &[(500, 5.0)]);

    sync(&app, character).await;

    assert_eq!(writes(&app, "DELETE").len(), 1);
    assert_json_eq(&serde_json::json!([]), &ledger(&app, character).await);
}

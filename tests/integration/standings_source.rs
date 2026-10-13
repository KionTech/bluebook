//! tests/Feature/Standings/StandingsSourceServiceTest.php: reading the
//! source's contacts into the canonical standings, storing their entities
//! and closing requests that got a blue standing.

use serde_json::json;

use crate::common::factories::{NewCharacter, NewRequest};
use crate::common::{
    READ_ALLIANCE_CONTACTS, READ_CHARACTER_CONTACTS, READ_CORPORATION_CONTACTS, TestApp,
    assert_json_eq,
};

/// `adminReader()`: an admin character that can read a source.
async fn admin_reader(app: &mut TestApp, kind: &str, entity_id: i64, scope: &str) -> i64 {
    let mut new = NewCharacter::default();
    match kind {
        "corporation" => {
            app.corporation(entity_id, None, None, None).await;
            new.corporation_id = Some(entity_id);
        }
        "alliance" => {
            app.alliance(entity_id, None, None).await;
            new.alliance_id = Some(entity_id);
        }
        _ => new.id = Some(entity_id),
    }
    let admin = app.character(new).await;
    app.grant_scopes(admin, &[scope]).await;
    app.admins(&[admin]);
    admin
}

async fn refresh(app: &TestApp) -> Option<bool> {
    bluebook::standings::source::refresh(&app.state).await
}

async fn contact_name(app: &TestApp, id: i64) -> serde_json::Value {
    app.scalar(&format!(
        "select coalesce(
            (select name from characters where id = c.contact_id and c.contact_type = 'character'),
            (select name from corporations where id = c.contact_id and c.contact_type = 'corporation'),
            (select name from alliances where id = c.contact_id and c.contact_type = 'alliance'))
         from source_contacts c where contact_id = {id}"
    ))
    .await
}

#[tokio::test]
async fn fetches_and_stores_corporation_source_contacts() {
    let mut app = TestApp::new().await;
    admin_reader(&mut app, "corporation", 2000, READ_CORPORATION_CONTACTS).await;
    app.source("corporation", 2000).await;
    app.esi.contacts(
        "corporations/2000",
        &[(100, "alliance", 5.0), (200, "character", -10.0)],
    );
    app.esi.name(100, "alliance", "Alpha Alliance");
    app.esi.name(200, "character", "Bravo Pilot");
    app.esi.affiliation(200, 2001, Some(100));
    app.esi.corporation(2001, "Bravo Corp", Some(100));
    app.esi.alliance(100, "Alpha Alliance");

    assert_eq!(refresh(&app).await, Some(true));

    assert_eq!(app.count("source_contacts").await, 2);
    assert_eq!(
        app.scalar("select standing::float8 from source_contacts where contact_id = 100")
            .await,
        json!(5.0)
    );
    assert_eq!(contact_name(&app, 100).await, json!("Alpha Alliance"));
    assert_eq!(contact_name(&app, 200).await, json!("Bravo Pilot"));
    assert_eq!(
        app.scalar("select last_synced_at is not null from standings_sources")
            .await,
        json!(true)
    );
}

#[tokio::test]
async fn stores_contact_entities_locally_with_their_affiliations() {
    let mut app = TestApp::new().await;
    admin_reader(&mut app, "corporation", 2000, READ_CORPORATION_CONTACTS).await;
    app.source("corporation", 2000).await;
    app.esi.contacts(
        "corporations/2000",
        &[
            (95_000_300, "character", 10.0),
            (2001, "corporation", 10.0),
            (100, "alliance", 10.0),
        ],
    );
    app.esi.name(95_000_300, "character", "Member Pilot");
    app.esi.affiliation(95_000_300, 2001, Some(100));
    app.esi.corporation(2001, "Member Corp", Some(100));
    app.esi.alliance(100, "Alpha Alliance");

    refresh(&app).await;

    assert_json_eq(
        &json!([{ "name": "Member Pilot", "corporation_id": 2001, "alliance_id": 100 }]),
        &json!(
            app.rows(
                "select name, corporation_id, alliance_id from characters where id = 95000300"
            )
            .await
        ),
    );
    assert_eq!(
        app.scalar("select alliance_id from corporations where id = 2001")
            .await,
        json!(100)
    );
    assert_eq!(
        app.scalar("select name from alliances where id = 100")
            .await,
        json!("Alpha Alliance")
    );
}

#[tokio::test]
async fn removes_source_contacts_the_source_no_longer_has() {
    let mut app = TestApp::new().await;
    admin_reader(&mut app, "alliance", 3000, READ_ALLIANCE_CONTACTS).await;
    app.source("alliance", 3000).await;
    app.source_contact(999, "character", 10.0).await;
    app.esi
        .contacts("alliances/3000", &[(100, "character", 5.0)]);
    app.esi.name(100, "character", "Charlie Pilot");
    app.esi.affiliation(100, 2001, None);
    app.esi.corporation(2001, "Charlie Corp", None);

    refresh(&app).await;

    assert_json_eq(
        &json!([{ "contact_id": 100 }]),
        &json!(app.rows("select contact_id from source_contacts").await),
    );
}

#[tokio::test]
async fn does_nothing_when_no_source_is_configured() {
    let mut app = TestApp::new().await;
    admin_reader(&mut app, "character", 95_000_001, READ_CHARACTER_CONTACTS).await;

    assert_eq!(refresh(&app).await, None);
    assert_eq!(app.esi.requests().len(), 0);
}

#[tokio::test]
async fn does_nothing_when_no_admin_can_read_the_source() {
    let mut app = TestApp::new().await;
    admin_reader(&mut app, "corporation", 2000, READ_CHARACTER_CONTACTS).await;
    app.source("corporation", 2000).await;

    assert_eq!(refresh(&app).await, None);
    assert_eq!(app.esi.requests().len(), 0);
}

/// A corporation source, a pending request for a character, and the
/// character's standing in the source.
async fn requested(app: &mut TestApp, standing: f64) -> i64 {
    admin_reader(app, "corporation", 2000, READ_CORPORATION_CONTACTS).await;
    app.source("corporation", 2000).await;
    let character = app.character(NewCharacter::default()).await;
    let request = app
        .standing_request(NewRequest {
            subject_type: Some("character"),
            subject_id: Some(character),
            requested_by: Some(character),
            status: Some("pending"),
            ..NewRequest::default()
        })
        .await;
    app.esi
        .contacts("corporations/2000", &[(character, "character", standing)]);
    app.esi.name(character, "character", "Requested Pilot");
    app.esi.affiliation(character, 2001, None);
    app.esi.corporation(2001, "Requested Corp", None);
    request
}

#[tokio::test]
async fn marks_a_pending_request_done_when_a_direct_blue_standing_appears() {
    let mut app = TestApp::new().await;
    let request = requested(&mut app, 5.0).await;

    refresh(&app).await;

    assert_eq!(app.request_status(request).await, "done");
}

#[tokio::test]
async fn does_not_close_a_request_when_the_direct_standing_is_neutral_or_red() {
    let mut app = TestApp::new().await;
    let request = requested(&mut app, 0.0).await;

    refresh(&app).await;

    assert_eq!(app.request_status(request).await, "pending");
}

#[tokio::test]
async fn does_not_auto_close_a_request_that_is_only_blue_through_a_parent() {
    let mut app = TestApp::new().await;
    admin_reader(&mut app, "corporation", 2000, READ_CORPORATION_CONTACTS).await;
    app.source("corporation", 2000).await;
    app.corporation(4000, Some("Member Corp"), None, None).await;
    let character = app
        .character(NewCharacter {
            corporation_id: Some(4000),
            ..NewCharacter::default()
        })
        .await;
    let request = app
        .standing_request(NewRequest {
            subject_type: Some("character"),
            subject_id: Some(character),
            requested_by: Some(character),
            status: Some("pending"),
            ..NewRequest::default()
        })
        .await;
    app.esi
        .contacts("corporations/2000", &[(4000, "corporation", 10.0)]);
    app.esi.name(4000, "corporation", "Member Corp");
    app.esi.corporation(4000, "Member Corp", None);

    refresh(&app).await;

    assert_eq!(app.request_status(request).await, "pending");
}

#[tokio::test]
async fn reports_no_change_and_skips_name_resolution_when_the_standings_are_unchanged() {
    let mut app = TestApp::new().await;
    admin_reader(&mut app, "corporation", 2000, READ_CORPORATION_CONTACTS).await;
    app.source("corporation", 2000).await;
    app.source_contact(100, "character", 5.0).await;
    app.esi
        .contacts("corporations/2000", &[(100, "character", 5.0)]);

    assert_eq!(refresh(&app).await, Some(false));

    // No name resolution when nothing changed, to save requests.
    assert!(
        app.esi
            .requests()
            .iter()
            .all(|request| request.path != "/universe/names/")
    );
}

// New.

#[tokio::test]
async fn reads_every_page_of_the_contacts() {
    let mut app = TestApp::new().await;
    admin_reader(&mut app, "corporation", 2000, READ_CORPORATION_CONTACTS).await;
    app.source("corporation", 2000).await;
    app.esi.page_size(2);
    app.esi.contacts(
        "corporations/2000",
        &[
            (1, "faction", 5.0),
            (2, "faction", 5.0),
            (3, "faction", -5.0),
            (4, "faction", 10.0),
            (5, "faction", 0.0),
        ],
    );

    assert_eq!(refresh(&app).await, Some(true));

    assert_eq!(
        app.esi.lines(),
        vec![
            "GET /corporations/2000/contacts/?page=1",
            "GET /corporations/2000/contacts/?page=2",
            "GET /corporations/2000/contacts/?page=3",
        ]
    );
    assert_eq!(app.count("source_contacts").await, 5);
}

/// FIXES.md, "Orderings without an ORDER BY".
#[tokio::test]
async fn the_first_admin_by_id_reads_the_source() {
    let mut app = TestApp::new().await;
    app.corporation(2000, None, None, None).await;
    let mut admins = Vec::new();
    for id in [95_000_002, 95_000_001] {
        let admin = app
            .character(NewCharacter {
                id: Some(id),
                corporation_id: Some(2000),
                ..NewCharacter::default()
            })
            .await;
        app.grant_scopes(admin, &[READ_CORPORATION_CONTACTS]).await;
        admins.push(admin);
    }
    app.admins(&admins);
    app.source("corporation", 2000).await;

    refresh(&app).await;

    let tokens: Vec<String> = app
        .rows("select access_token from esi_tokens where character_id = 95000001")
        .await
        .into_iter()
        .map(|row| row["access_token"].as_str().expect("a token").to_owned())
        .collect();
    assert_eq!(
        app.esi.requests()[0].header("authorization"),
        Some(format!("Bearer {}", tokens[0]).as_str())
    );
}

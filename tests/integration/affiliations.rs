//! tests/Feature/Console/SyncCharacterAffiliationsCommandTest.php: the
//! hourly refresh of the characters' and contacts' corporations and
//! alliances.

use serde_json::json;
use time::macros::datetime;

use crate::common::factories::NewCharacter;
use crate::common::mocks::esi_corporation_payload;
use crate::common::{TestApp, assert_json_eq};

#[tokio::test]
async fn refreshes_the_affiliations_of_user_characters_and_source_contacts() {
    let app = TestApp::new().await;
    // A user character whose corporation changed in game.
    app.corporation(2001, Some("Old Corp"), None, None).await;
    let user_character = app
        .character(NewCharacter {
            corporation_id: Some(2001),
            ..NewCharacter::default()
        })
        .await;
    // A contact character the app has never seen locally.
    app.source_contact(95_000_500, "character", 10.0).await;
    // A contact corporation that joined an alliance since it was stored.
    app.corporation(2002, Some("Member Corp"), None, None).await;
    app.source_contact(2002, "corporation", 10.0).await;
    // A contact alliance not yet known locally.
    app.source_contact(100, "alliance", 10.0).await;

    app.esi.affiliation(user_character, 2003, None);
    app.esi.affiliation(95_000_500, 2002, Some(100));
    app.esi
        .corporation_json(2002, esi_corporation_payload("Member Corp", Some(100)));
    app.esi
        .corporation_json(2003, esi_corporation_payload("New Corp", None));
    app.esi.alliance(100, "Alpha Alliance");

    assert_eq!(bluebook::entities::sync_affiliations(&app.state).await, 0);

    assert_json_eq(
        &json!([
            { "id": 95_000_500, "corporation_id": 2002, "alliance_id": 100 },
            { "id": user_character, "corporation_id": 2003, "alliance_id": null },
        ]),
        &json!(
            app.rows(&format!(
                "select id, corporation_id, alliance_id from characters
             where id in (95000500, {user_character}) order by id desc"
            ))
            .await
        ),
    );
    assert_json_eq(
        &json!([{ "alliance_id": 100 }]),
        &json!(
            app.rows("select alliance_id from corporations where id = 2002")
                .await
        ),
    );
    assert_json_eq(
        &json!([{ "name": "Alpha Alliance" }]),
        &json!(app.rows("select name from alliances where id = 100").await),
    );
}

#[tokio::test]
async fn skips_the_affiliation_sync_during_eve_downtime() {
    let app = TestApp::at(datetime!(2026-06-25 11:05:00 UTC)).await;
    app.character(NewCharacter::default()).await;

    assert_eq!(bluebook::entities::sync_affiliations(&app.state).await, 0);

    assert_eq!(app.esi.requests().len(), 0);
}

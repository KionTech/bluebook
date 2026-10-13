//! The admin settings page.

use crate::common::factories::{NewCharacter, Owner};
use crate::common::{READ_CORPORATION_CONTACTS, SEND_MAIL, TestApp};

#[tokio::test]
async fn shows_the_settings_page_with_source_options_and_discord_settings() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.admins(&[character]);

    let response = app.page(&app.sign_in(user).await, "/admin/settings").await;
    response.assert_status(200);
    let props = response.json();
    assert_eq!(props["sourceTypes"].as_array().map(Vec::len), Some(3));
    assert!(props["discordSettings"].is_object());
    assert!(props["adminCharacter"].is_object());
}

#[tokio::test]
async fn forbids_non_admins_from_the_settings_page() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    app.admins(&[999_999]);

    app.page(&app.sign_in(user).await, "/admin/settings")
        .await
        .assert_status(403);
}

#[tokio::test]
async fn offers_the_admin_characters_entities_while_an_alt_is_active() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    app.corporation(2000, Some("Admin Corp"), Some("ADMC"), None)
        .await;
    let admin = app
        .character(NewCharacter {
            name: Some("Admin Pilot"),
            owner: Owner::User(user),
            corporation_id: Some(2000),
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(admin, &[READ_CORPORATION_CONTACTS, SEND_MAIL])
        .await;
    let alt = app
        .character(NewCharacter {
            owner: Owner::User(user),
            ..NewCharacter::default()
        })
        .await;
    app.admins(&[admin]);

    let props = app
        .page(&app.sign_in_as(user, alt).await, "/admin/settings")
        .await
        .json();

    assert_eq!(props["sourceTypes"][1]["entity_id"], 2000);
    assert_eq!(props["sourceTypes"][1]["entity_name"], "Admin Corp");
    assert_eq!(props["sourceTypes"][1]["available"], true);
    assert_eq!(props["sourceTypes"][0]["entity_id"], admin);
    assert_eq!(
        props["adminCharacter"],
        serde_json::json!({ "id": admin, "name": "Admin Pilot", "has_mail_scope": true })
    );
}

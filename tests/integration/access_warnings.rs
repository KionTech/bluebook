//! tests/Feature/AccessWarningsTest.php: the banners for characters that
//! need a new token and for an unreadable source, from the shared `auth`
//! props.

use serde_json::json;

use crate::common::factories::{NewCharacter, Owner};
use crate::common::{TestApp, WRITE_CHARACTER_CONTACTS, assert_json_eq};

#[tokio::test]
async fn flags_opted_in_characters_missing_the_write_contacts_token() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let id = app
        .character(NewCharacter {
            name: Some("Tokenless"),
            owner: Owner::User(user),
            should_sync: true,
            ..NewCharacter::default()
        })
        .await;

    let props = app.page(&app.sign_in(user).await, "/dashboard").await;
    props.assert_status(200);
    assert_json_eq(
        &json!([{ "id": id, "name": "Tokenless" }]),
        &props.json()["auth"]["reauth_characters"],
    );
}

#[tokio::test]
async fn does_not_flag_a_scopeless_character_that_never_opted_into_syncing() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character(NewCharacter {
        owner: Owner::User(user),
        should_sync: false,
        ..NewCharacter::default()
    })
    .await;

    let props = app.page(&app.sign_in(user).await, "/dashboard").await;
    assert_json_eq(&json!([]), &props.json()["auth"]["reauth_characters"]);
}

#[tokio::test]
async fn does_not_flag_a_character_that_has_the_write_contacts_token() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app
        .character(NewCharacter {
            owner: Owner::User(user),
            should_sync: true,
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(character, &[WRITE_CHARACTER_CONTACTS])
        .await;

    let props = app.page(&app.sign_in(user).await, "/dashboard").await;
    assert_json_eq(&json!([]), &props.json()["auth"]["reauth_characters"]);
}

#[tokio::test]
async fn flags_an_unreadable_source_for_admins() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.grant_scopes(character, &[WRITE_CHARACTER_CONTACTS])
        .await;
    app.admins(&[character]);
    app.source("corporation", 2000).await;

    let props = app.page(&app.sign_in(user).await, "/dashboard").await;
    assert_eq!(props.json()["auth"]["source_unreadable"], true);
}

#[tokio::test]
async fn does_not_flag_the_source_for_non_admins() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.grant_scopes(character, &[WRITE_CHARACTER_CONTACTS])
        .await;
    app.source("corporation", 2000).await;

    let props = app.page(&app.sign_in(user).await, "/dashboard").await;
    assert_eq!(props.json()["auth"]["source_unreadable"], false);
}

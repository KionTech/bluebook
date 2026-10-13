//! Who may see the source's standings on the dashboard.

use serde_json::json;

use crate::common::factories::{NewCharacter, Owner};
use crate::common::{Browser, TestApp};

async fn dashboard(app: &TestApp, browser: &Browser) -> serde_json::Value {
    let response = app.page(browser, "/dashboard").await;
    response.assert_status(200);
    response.json()
}

#[tokio::test]
async fn hides_the_standings_from_users_without_an_eligible_character() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    app.source("character", 999).await;
    app.source_contact(90_100_001, "character", 10.0).await;
    app.source_contact(90_100_002, "corporation", 10.0).await;

    let props = dashboard(&app, &app.sign_in(user).await).await;
    assert_eq!(props["canViewStandings"], false);
    assert_eq!(props["standings"], json!(null));
}

#[tokio::test]
async fn shows_the_standings_to_a_user_whose_character_is_covered_by_the_source() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.corporation(2000, Some("Source Corp"), None, None).await;
    app.character(NewCharacter {
        owner: Owner::User(user),
        corporation_id: Some(2000),
        ..NewCharacter::default()
    })
    .await;
    app.source("corporation", 2000).await;
    app.any_source_contact().await;

    let props = dashboard(&app, &app.sign_in(user).await).await;
    assert_eq!(props["canViewStandings"], true);
    assert_eq!(props["standings"].as_array().map(Vec::len), Some(1));
}

#[tokio::test]
async fn shows_the_standings_to_a_user_with_a_directly_blue_character() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.source("character", 999).await;
    app.source_contact(character, "character", 5.0).await;

    let props = dashboard(&app, &app.sign_in(user).await).await;
    assert_eq!(props["canViewStandings"], true);
}

#[tokio::test]
async fn shows_the_standings_to_a_user_blue_via_their_alliance() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.alliance(3000, Some("Blue Alliance"), None).await;
    app.character(NewCharacter {
        owner: Owner::User(user),
        alliance_id: Some(3000),
        ..NewCharacter::default()
    })
    .await;
    app.source("character", 999).await;
    app.source_contact(3000, "alliance", 10.0).await;

    let props = dashboard(&app, &app.sign_in(user).await).await;
    assert_eq!(props["canViewStandings"], true);
}

#[tokio::test]
async fn keeps_the_standings_hidden_from_a_red_character() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.source("character", 999).await;
    app.source_contact(character, "character", -10.0).await;

    let props = dashboard(&app, &app.sign_in(user).await).await;
    assert_eq!(props["canViewStandings"], false);
    assert_eq!(props["standings"], json!(null));
}

#[tokio::test]
async fn always_shows_the_standings_to_admins() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.admins(&[character]);
    app.source("character", 999).await;
    app.any_source_contact().await;

    let props = dashboard(&app, &app.sign_in(user).await).await;
    assert_eq!(props["canViewStandings"], true);
}

#[tokio::test]
async fn still_offers_request_options_to_ineligible_users() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    app.source("character", 999).await;

    let props = dashboard(&app, &app.sign_in(user).await).await;
    assert_eq!(props["canViewStandings"], false);
    assert_eq!(props["requestableOptions"][0]["type"], "character");
}

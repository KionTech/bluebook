//! The dashboard's standings with their redundancy flags, the characters
//! and the request options.

use serde_json::{Value, json};

use crate::common::factories::{NewCharacter, Owner};
use crate::common::{APP_URL, Browser, TestApp, assert_json_eq};

/// A user whose character is in the source alliance 3000.
async fn source_member(app: &TestApp) -> Browser {
    let user = app.user().await;
    app.alliance(3000, Some("Home Alliance"), None).await;
    app.character(NewCharacter {
        owner: Owner::User(user),
        alliance_id: Some(3000),
        ..NewCharacter::default()
    })
    .await;
    app.source("alliance", 3000).await;
    app.sign_in(user).await
}

async fn standings(app: &TestApp, browser: &Browser) -> Value {
    let response = app.page(browser, "/dashboard").await;
    response.assert_status(200);
    response.json()["standings"].clone()
}

#[tokio::test]
async fn shows_the_source_standings_and_the_user_characters_on_the_dashboard() {
    let app = TestApp::new().await;
    let browser = source_member(&app).await;
    app.any_source_contact().await;
    app.any_source_contact().await;

    let props = app.page(&browser, "/dashboard").await.json();
    assert_eq!(props["canViewStandings"], true);
    assert_eq!(props["standings"].as_array().map(Vec::len), Some(2));
    assert_eq!(props["characters"].as_array().map(Vec::len), Some(1));
}

/// A blue alliance 4000 and a member character of it.
async fn blue_alliance_member(app: &TestApp) -> i64 {
    app.alliance(4000, Some("Blue Alliance"), None).await;
    app.character(NewCharacter {
        alliance_id: Some(4000),
        ..NewCharacter::default()
    })
    .await
}

#[tokio::test]
async fn flags_a_character_standing_as_redundant_when_its_alliance_carries_the_same_standing() {
    let app = TestApp::new().await;
    let browser = source_member(&app).await;
    let member = blue_alliance_member(&app).await;
    app.source_contact(4000, "alliance", 10.0).await;
    app.source_contact(member, "character", 10.0).await;

    let standings = standings(&app, &browser).await;
    assert_eq!(standings[0]["redundant_via"], json!(null));
    assert_json_eq(
        &json!({ "contact_type": "alliance", "contact_id": 4000, "name": "Blue Alliance" }),
        &standings[1]["redundant_via"],
    );
}

#[tokio::test]
async fn flags_a_corporation_standing_as_redundant_when_its_alliance_carries_the_same_standing() {
    let app = TestApp::new().await;
    let browser = source_member(&app).await;
    app.alliance(4000, Some("Blue Alliance"), None).await;
    app.corporation(2000, Some("Blue Corp"), None, Some(4000))
        .await;
    app.source_contact(4000, "alliance", 5.0).await;
    app.source_contact(2000, "corporation", 5.0).await;

    let standings = standings(&app, &browser).await;
    assert_eq!(standings[1]["redundant_via"]["contact_type"], "alliance");
    assert_eq!(standings[1]["redundant_via"]["contact_id"], 4000);
}

/// `flags a weaker standing that its parent covers in the same direction`
/// and `does not flag a standing its parent does not cover`.
async fn parent_and_member(parent: f64, member_standing: f64, flagged: bool) {
    let app = TestApp::new().await;
    let browser = source_member(&app).await;
    let member = blue_alliance_member(&app).await;
    app.source_contact(4000, "alliance", parent).await;
    app.source_contact(member, "character", member_standing)
        .await;

    // Standings sort by value descending, so a weaker negative child
    // appears above its more negative parent.
    let member_index = if member_standing > parent { 0 } else { 1 };
    let standings = standings(&app, &browser).await;
    if flagged {
        assert_eq!(standings[1 - member_index]["redundant_via"], json!(null));
        assert_eq!(standings[member_index]["redundant_via"]["contact_id"], 4000);
    } else {
        assert_eq!(standings[member_index]["redundant_via"], json!(null));
    }
}

#[tokio::test]
async fn flags_a_weaker_standing_positive_parent_covers_weaker_positive() {
    parent_and_member(10.0, 5.0, true).await;
}

#[tokio::test]
async fn flags_a_weaker_standing_negative_parent_covers_weaker_negative() {
    parent_and_member(-10.0, -5.0, true).await;
}

#[tokio::test]
async fn does_not_flag_a_standing_stronger_positive_than_parent() {
    parent_and_member(5.0, 10.0, false).await;
}

#[tokio::test]
async fn does_not_flag_a_standing_stronger_negative_than_parent() {
    parent_and_member(-5.0, -10.0, false).await;
}

#[tokio::test]
async fn does_not_flag_a_standing_opposite_direction_of_parent() {
    parent_and_member(-10.0, 5.0, false).await;
}

#[tokio::test]
async fn does_not_flag_a_standing_neutral_under_a_blue_parent() {
    parent_and_member(10.0, 0.0, false).await;
}

#[tokio::test]
async fn prefers_the_corporation_over_the_alliance_when_both_make_a_character_redundant() {
    let app = TestApp::new().await;
    let browser = source_member(&app).await;
    app.alliance(4000, Some("Blue Alliance"), None).await;
    app.corporation(2000, Some("Blue Corp"), None, Some(4000))
        .await;
    let member = app
        .character(NewCharacter {
            corporation_id: Some(2000),
            alliance_id: Some(4000),
            ..NewCharacter::default()
        })
        .await;
    app.source_contact(4000, "alliance", 10.0).await;
    app.source_contact(2000, "corporation", 10.0).await;
    app.source_contact(member, "character", 10.0).await;

    let standings = standings(&app, &browser).await;
    assert_eq!(standings[1]["redundant_via"]["contact_type"], "corporation");
    assert_eq!(standings[1]["redundant_via"]["contact_id"], 2000);
}

#[tokio::test]
async fn offers_a_request_option_for_a_character_with_no_standing() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    app.source("character", 999).await;

    let props = app
        .page(&app.sign_in(user).await, "/dashboard")
        .await
        .json();
    assert_eq!(props["requestableOptions"][0]["type"], "character");
    assert_eq!(props["requestableOptions"][0]["status"], json!(null));
}

#[tokio::test]
async fn exposes_the_corporation_entity_as_a_deduplicated_request_option() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.corporation(2000, Some("Blue Corp"), None, None).await;
    let character = app
        .character(NewCharacter {
            owner: Owner::User(user),
            corporation_id: Some(2000),
            ..NewCharacter::default()
        })
        .await;
    app.source("character", 999).await;

    let props = app
        .page(&app.sign_in(user).await, "/dashboard")
        .await
        .json();
    assert_eq!(props["requestableOptions"][1]["type"], "corporation");
    assert_eq!(props["requestableOptions"][1]["id"], 2000);
    assert_eq!(
        props["requestableOptions"][1]["via_character_id"],
        character
    );
}

#[tokio::test]
async fn guests_are_redirected_to_the_login_page() {
    let app = TestApp::new().await;

    let response = app.page(&Browser::guest(), "/dashboard").await;

    response.assert_status(302);
    assert_eq!(response.location(), format!("{APP_URL}/eve"));
}

#[tokio::test]
async fn authenticated_users_can_visit_the_dashboard() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;

    app.page(&app.sign_in(user).await, "/dashboard")
        .await
        .assert_status(200);
}

// Orderings the pages rely on.

#[tokio::test]
async fn characters_are_listed_by_id() {
    let app = TestApp::new().await;
    let user = app.user().await;
    for id in [95_000_003, 95_000_001, 95_000_002] {
        app.character(NewCharacter {
            id: Some(id),
            owner: Owner::User(user),
            ..NewCharacter::default()
        })
        .await;
    }

    let props = app
        .page(&app.sign_in(user).await, "/dashboard")
        .await
        .json();
    let ids = |list: &Value| -> Vec<i64> {
        list.as_array()
            .expect("a list")
            .iter()
            .map(|item| item["id"].as_i64().expect("an id"))
            .collect()
    };
    assert_eq!(
        ids(&props["characters"]),
        vec![95_000_001, 95_000_002, 95_000_003]
    );
    assert_eq!(
        ids(&props["auth"]["characters"]),
        vec![95_000_001, 95_000_002, 95_000_003]
    );
    // Without an active character in the session, the first one is.
    assert_eq!(props["auth"]["active_character"]["id"], 95_000_001);
}

#[tokio::test]
async fn a_user_without_characters_is_signed_out() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let browser = app.sign_in(user).await;

    let response = app.page(&browser, "/dashboard").await;

    response.assert_status(403);
    assert_eq!(
        response.json()["message"],
        "No characters found. Please log in again."
    );
    assert_eq!(app.count("sessions").await, 0);
}

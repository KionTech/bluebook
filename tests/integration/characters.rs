//! The pilot's character forms: tests/Feature/MainCharacterTest.php,
//! tests/Feature/Standings/CharacterSyncToggleTest.php and
//! tests/Feature/UserCharacterTest.php.

use serde_json::json;

use crate::common::factories::{NewCharacter, Owner};
use crate::common::{APP_URL, Browser, TestApp, WRITE_CHARACTER_CONTACTS, assert_json_eq};

// MainCharacterTest.php

#[tokio::test]
async fn users_can_set_their_main_character() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    let main = app.character_of(user).await;

    app.put(
        &app.sign_in(user).await,
        &format!("/auth/character/{main}/main"),
        json!({}),
    )
    .await
    .assert_done();

    assert_eq!(app.main_character_of(user).await, Some(main));
}

#[tokio::test]
async fn users_cannot_set_a_character_they_do_not_own_as_main() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    let other = app.character(NewCharacter::default()).await;

    app.put(
        &app.sign_in(user).await,
        &format!("/auth/character/{other}/main"),
        json!({}),
    )
    .await
    .assert_status(403);

    assert_eq!(app.main_character_of(user).await, None);
}

#[tokio::test]
async fn removing_the_main_character_clears_the_main_selection() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    let main = app.character_of(user).await;
    app.set_main(user, main).await;

    app.delete(&app.sign_in(user).await, &format!("/auth/character/{main}"))
        .await
        .assert_done();

    assert_eq!(app.main_character_of(user).await, None);
}

#[tokio::test]
async fn removing_an_alt_keeps_the_main_selection() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let main = app.character_of(user).await;
    let alt = app.character_of(user).await;
    app.set_main(user, main).await;

    app.delete(&app.sign_in(user).await, &format!("/auth/character/{alt}"))
        .await
        .assert_done();

    assert_eq!(app.main_character_of(user).await, Some(main));
}

#[tokio::test]
async fn the_dashboard_marks_the_main_character() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let main = app.character_of(user).await;
    app.character_of(user).await;
    app.set_main(user, main).await;

    let response = app.page(&app.sign_in(user).await, "/dashboard").await;
    response.assert_status(200);
    let props = response.json();
    assert!(
        props["characters"]
            .as_array()
            .expect("characters")
            .iter()
            .any(|character| character["id"] == main && character["is_main"] == true)
    );
}

// CharacterSyncToggleTest.php

#[tokio::test]
async fn lets_a_user_re_sync_only_their_own_opted_in_characters() {
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
    let other = app
        .character(NewCharacter {
            should_sync: true,
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(other, &[WRITE_CHARACTER_CONTACTS]).await;

    app.post(&app.sign_in(user).await, "/standings/sync", json!({}))
        .await
        .assert_done();

    assert_json_eq(
        &json!([{ "character_id": character }]),
        &json!(app.jobs_of("character_sync").await),
    );
}

#[tokio::test]
async fn lets_a_user_enable_syncing_for_their_own_character() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;

    app.put(
        &app.sign_in(user).await,
        &format!("/standings/{character}"),
        json!({ "should_sync": true }),
    )
    .await
    .assert_done();

    assert!(app.should_sync(character).await);
}

#[tokio::test]
async fn lets_a_user_disable_syncing_for_their_own_character() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app
        .character(NewCharacter {
            owner: Owner::User(user),
            should_sync: true,
            ..NewCharacter::default()
        })
        .await;

    app.put(
        &app.sign_in(user).await,
        &format!("/standings/{character}"),
        json!({ "should_sync": false }),
    )
    .await
    .assert_done();

    assert!(!app.should_sync(character).await);
}

#[tokio::test]
async fn forbids_toggling_a_character_belonging_to_another_user() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    let other = app.character(NewCharacter::default()).await;

    app.put(
        &app.sign_in(user).await,
        &format!("/standings/{other}"),
        json!({ "should_sync": true }),
    )
    .await
    .assert_status(403);

    assert!(!app.should_sync(other).await);
}

#[tokio::test]
async fn requires_authentication_to_toggle_syncing() {
    let app = TestApp::new().await;
    let character = app.character(NewCharacter::default()).await;

    let response = app
        .put(
            &Browser::guest(),
            &format!("/standings/{character}"),
            json!({ "should_sync": true }),
        )
        .await;

    // Laravel redirected to the login; the form answers 401 with it
    // (FIXES.md, "Cross-site forms are refused without a CSRF token").
    response.assert_status(401);
    assert_json_eq(
        &json!({ "message": "Unauthenticated.", "redirect": "/eve" }),
        &response.json(),
    );
}

// UserCharacterTest.php

#[tokio::test]
async fn users_can_switch_their_active_character() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let first = app.character_of(user).await;
    let second = app.character_of(user).await;
    let browser = app.sign_in_as(user, first).await;

    app.put(&browser, &format!("/auth/character/{second}"), json!({}))
        .await
        .assert_done();

    let session = app.session_data(&browser).await.expect("a session");
    assert_eq!(session["active_character_id"], second);
}

#[tokio::test]
async fn users_cannot_switch_to_a_character_they_do_not_own() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    let other = app.character(NewCharacter::default()).await;

    app.put(
        &app.sign_in(user).await,
        &format!("/auth/character/{other}"),
        json!({}),
    )
    .await
    .assert_status(403);
}

#[tokio::test]
async fn users_can_remove_a_character_from_their_account() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let first = app.character_of(user).await;
    let second = app.character_of(user).await;
    let browser = app.sign_in_as(user, first).await;

    app.delete(&browser, &format!("/auth/character/{second}"))
        .await
        .assert_done();

    assert!(app.user_exists(user).await);
    assert_eq!(app.owner_of(second).await, None);
}

#[tokio::test]
async fn removing_last_character_deletes_the_user_account() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;

    let redirect = app
        .delete(
            &app.sign_in(user).await,
            &format!("/auth/character/{character}"),
        )
        .await
        .assert_done();

    assert_eq!(redirect, Some("/".to_owned()));
    assert!(!app.user_exists(user).await);
    assert_eq!(app.owner_of(character).await, None);
}

#[tokio::test]
async fn users_cannot_remove_a_character_they_do_not_own() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    let other = app.character(NewCharacter::default()).await;

    app.delete(
        &app.sign_in(user).await,
        &format!("/auth/character/{other}"),
    )
    .await
    .assert_status(403);
}

#[tokio::test]
async fn active_character_switches_to_remaining_character_after_removal() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let first = app.character_of(user).await;
    let second = app.character_of(user).await;
    let browser = app.sign_in_as(user, first).await;

    app.delete(&browser, &format!("/auth/character/{first}"))
        .await
        .assert_done();

    let session = app.session_data(&browser).await.expect("a session");
    assert_eq!(session["active_character_id"], second);
}

// New.

/// FIXES.md, "Removed characters were mailed about lapsed tokens
/// forever".
#[tokio::test]
async fn removing_a_character_turns_its_sync_off() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    let synced = app
        .character(NewCharacter {
            owner: Owner::User(user),
            should_sync: true,
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(synced, &[WRITE_CHARACTER_CONTACTS]).await;

    app.delete(
        &app.sign_in(user).await,
        &format!("/auth/character/{synced}"),
    )
    .await
    .assert_done();

    assert_json_eq(
        &json!([{ "user_id": null, "character_owner_hash": null, "should_sync": false }]),
        &json!(app.rows(&format!(
            "select user_id, character_owner_hash, should_sync from characters where id = {synced}"
        )).await),
    );
    assert_eq!(app.count("esi_tokens").await, 0);
}

#[tokio::test]
async fn removing_the_last_character_signs_out_and_flashes_on_the_home_page() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    let browser = app.sign_in(user).await;

    let response = app
        .delete(&browser, &format!("/auth/character/{character}"))
        .await;
    response.assert_done();

    let next = response.browser(&browser);
    assert_ne!(next.cookie, browser.cookie);
    let props = app.page(&next, "/").await.json();
    assert_eq!(props["auth"]["user"], json!(null));
    assert_eq!(props["flash"]["success"], "Your account has been deleted.");
    assert!(APP_URL.starts_with("https://"));
}

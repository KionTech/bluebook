//! tests/Feature/SetupWizardTest.php: the first-login wizard, shown until
//! a main character is picked or the wizard is skipped.

use serde_json::json;

use crate::common::TestApp;

#[tokio::test]
async fn the_setup_wizard_is_shown_to_a_user_without_a_main_character() {
    let app = TestApp::new().await;
    let user = app.user_without_setup().await;
    app.character_of(user).await;

    let response = app.page(&app.sign_in(user).await, "/dashboard").await;
    response.assert_status(200);
    assert_eq!(response.json()["showSetupWizard"], true);
}

#[tokio::test]
async fn the_setup_wizard_is_not_shown_once_a_main_character_is_selected() {
    let app = TestApp::new().await;
    let user = app.user_without_setup().await;
    let main = app.character_of(user).await;
    app.set_main(user, main).await;

    let props = app
        .page(&app.sign_in(user).await, "/dashboard")
        .await
        .json();
    assert_eq!(props["showSetupWizard"], false);
}

#[tokio::test]
async fn the_setup_wizard_is_not_shown_after_it_was_skipped() {
    let app = TestApp::new().await;
    let user = app.user_without_setup().await;
    app.character_of(user).await;
    let browser = app.sign_in(user).await;

    let redirect = app
        .post(&browser, "/setup/complete", json!({}))
        .await
        .assert_done();
    assert_eq!(redirect, None);

    assert_eq!(
        app.scalar(&format!(
            "select setup_completed_at is not null from users where id = {user}"
        ))
        .await,
        json!(true)
    );
    let props = app.page(&browser, "/dashboard").await.json();
    assert_eq!(props["showSetupWizard"], false);
}

#[tokio::test]
async fn selecting_a_main_character_completes_the_setup() {
    let app = TestApp::new().await;
    let user = app.user_without_setup().await;
    let main = app.character_of(user).await;

    app.put(
        &app.sign_in(user).await,
        &format!("/auth/character/{main}/main"),
        json!({}),
    )
    .await
    .assert_done();

    assert_eq!(app.main_character_of(user).await, Some(main));
    assert_eq!(
        app.scalar(&format!(
            "select setup_completed_at is not null from users where id = {user}"
        ))
        .await,
        json!(true)
    );
}

#[tokio::test]
async fn completing_the_setup_does_not_overwrite_an_earlier_completion_time() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    sqlx::query("update users set setup_completed_at = $2 where id = $1")
        .bind(user)
        .bind(app.clock.now() - time::Duration::days(1))
        .execute(&app.pool)
        .await
        .expect("a completion time");

    app.post(&app.sign_in(user).await, "/setup/complete", json!({}))
        .await
        .assert_done();

    assert_eq!(
        app.scalar(&format!(
            "select extract(epoch from setup_completed_at)::bigint from users where id = {user}"
        ))
        .await,
        json!((app.clock.now() - time::Duration::days(1)).unix_timestamp())
    );
}

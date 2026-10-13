//! tests/Feature/Admin/SettingsPageTest.php: the admin settings page.

use crate::common::TestApp;

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

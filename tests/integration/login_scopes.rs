//! The scopes the login asks EVE for, and the grant links that ask for
//! more.

use serde_json::json;

use crate::common::{APP_URL, Browser, SEND_MAIL, TestApp};

fn decoded(location: &str) -> String {
    form_urlencoded::parse(
        location
            .split_once('?')
            .map_or("", |(_, query)| query)
            .as_bytes(),
    )
    .map(|(key, value)| format!("{key}={value}"))
    .collect::<Vec<_>>()
    .join("&")
}

#[tokio::test]
async fn asks_for_no_scopes_at_a_normal_login() {
    let app = TestApp::new().await;

    let response = app.get(&Browser::guest(), "/eve").await;

    response.assert_status(302);
    let location = response.location();
    assert!(!decoded(location).contains("esi-"), "{location}");
    assert!(location.contains("&scope=&"), "{location}");
}

#[tokio::test]
async fn shares_a_sync_grant_link_asking_only_for_the_character_contact_scopes() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;

    let props = app
        .page(&app.sign_in(user).await, "/dashboard")
        .await
        .json();

    assert_eq!(
        props["auth"]["sync_scopes_url"],
        json!(format!(
            "{APP_URL}/eve?add_to_account=1&scopes=esi-characters.read_contacts.v1%2Cesi-characters.write_contacts.v1"
        ))
    );
}

#[tokio::test]
async fn shares_the_admin_grant_link_only_with_admins() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.admins(&[character]);
    let browser = app.sign_in(user).await;

    let props = app.page(&browser, "/dashboard").await.json();
    let url = props["auth"]["admin_scopes_url"].as_str().expect("a link");
    assert!(url.contains("add_to_account=1"), "{url}");
    for scope in [
        "esi-corporations.read_contacts.v1",
        "esi-alliances.read_contacts.v1",
        "esi-mail.send_mail.v1",
    ] {
        assert!(url.contains(scope), "{url}");
    }

    app.admins(&[]);
    let props = app.page(&browser, "/dashboard").await.json();
    assert_eq!(props["auth"]["admin_scopes_url"], json!(null));
}

#[tokio::test]
async fn asks_for_exactly_the_scopes_a_grant_link_carries() {
    let app = TestApp::new().await;

    let response = app
        .get(
            &Browser::guest(),
            "/eve?add_to_account=1&scopes=esi-mail.send_mail.v1%2Cesi-characters.write_contacts.v1",
        )
        .await;

    response.assert_status(302);
    let location = decoded(response.location());
    assert!(
        location.contains("scope=esi-mail.send_mail.v1 esi-characters.write_contacts.v1"),
        "{location}"
    );
    assert!(!location.contains("esi-corporations.read_contacts.v1"));
}

#[tokio::test]
async fn asks_for_the_mail_scope_through_the_admin_grant_link() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.admins(&[character]);

    let response = app.page(&app.sign_in(user).await, "/admin/settings").await;
    response.assert_status(200);
    let url = response.json()["grantMailScopeUrl"]
        .as_str()
        .expect("a link")
        .to_owned();
    assert!(url.contains("add_to_account=1"), "{url}");
    assert!(url.contains("esi-mail.send_mail.v1"), "{url}");
    assert!(url.contains("esi-characters.write_contacts.v1"), "{url}");
}

#[tokio::test]
async fn shows_whether_the_admin_character_can_send_expiry_mail() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.admins(&[character]);
    let browser = app.sign_in(user).await;

    let props = app.page(&browser, "/admin/settings").await.json();
    assert_eq!(props["adminCharacter"]["has_mail_scope"], false);

    app.grant_scopes(character, &[SEND_MAIL]).await;

    let props = app.page(&browser, "/admin/settings").await.json();
    assert_eq!(props["adminCharacter"]["has_mail_scope"], true);
}

#[tokio::test]
async fn an_unknown_scope_is_a_bad_request() {
    let app = TestApp::new().await;

    let response = app
        .get_with(
            &Browser::guest(),
            "/eve?scopes=nope",
            &[("accept", "text/html")],
        )
        .await;

    response.assert_status(400);
    assert!(response.text().contains("Bad Request"));
    assert_eq!(app.count("sessions").await, 0);
}

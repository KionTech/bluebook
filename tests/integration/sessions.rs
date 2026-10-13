//! The sign-in without Laravel's sessions (FIXES.md, "One new login
//! instead of a session bridge", "Cross-site forms are refused without a
//! CSRF token"): the session's lifetime, the same-origin check and a
//! guest's form.

use axum::body::Body;
use axum::http::Method;
use serde_json::json;
use time::Duration;

use crate::common::{APP_URL, Browser, HOST, TestApp, assert_json_eq, header_map};

#[tokio::test]
async fn a_session_lasts_its_lifetime() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    let browser = app.sign_in(user).await;

    app.clock
        .advance(Duration::days(400) - Duration::seconds(1));
    app.page(&browser, "/dashboard").await.assert_status(200);

    app.clock.advance(Duration::seconds(1));
    let response = app.page(&browser, "/dashboard").await;
    response.assert_status(302);
    assert_eq!(response.location(), format!("{APP_URL}/eve"));
}

#[tokio::test]
async fn a_laravel_session_cookie_is_a_guest() {
    let app = TestApp::new().await;
    let browser = Browser {
        cookie: Some("bluebook-session=eyJpdiI6IlUyZEwzdkl2UzFlQU1hL2JXTzR5RkE9PSJ9".to_owned()),
    };

    let props = app.page(&browser, "/").await.json();

    assert_eq!(props["auth"]["user"], json!(null));
}

#[tokio::test]
async fn a_cross_site_form_is_refused() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    let browser = app.sign_in(user).await;
    let cookie = browser.cookie.clone().expect("a cookie");

    for headers in [
        vec![("sec-fetch-site", "cross-site")],
        vec![("origin", "https://evil.example")],
        vec![("sec-fetch-site", "same-site"), ("origin", APP_URL)],
    ] {
        let mut all = vec![
            ("host", HOST),
            ("cookie", cookie.as_str()),
            ("content-type", "application/json"),
            ("accept", "application/json"),
        ];
        all.extend(headers.clone());
        let response = app
            .request(
                Method::PUT,
                &format!("/standings/{character}"),
                header_map(&all),
                Body::from(json!({ "should_sync": true }).to_string()),
            )
            .await;
        response.assert_status(419);
        assert_json_eq(
            &json!({ "message": "CSRF token mismatch." }),
            &response.json(),
        );
    }
    assert!(!app.should_sync(character).await);
}

#[tokio::test]
async fn a_form_without_browser_headers_passes() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    let browser = app.sign_in(user).await;

    let response = app
        .request(
            Method::PUT,
            &format!("/standings/{character}"),
            header_map(&[
                ("host", HOST),
                ("cookie", browser.cookie.as_deref().expect("a cookie")),
                ("content-type", "application/json"),
                ("accept", "application/json"),
            ]),
            Body::from(json!({ "should_sync": true }).to_string()),
        )
        .await;

    response.assert_done();
    assert!(app.should_sync(character).await);
}

#[tokio::test]
async fn a_guest_form_is_sent_to_the_login() {
    let app = TestApp::new().await;

    for (method, path) in [
        (Method::POST, "/standings/sync"),
        (Method::PUT, "/auth/character/1"),
        (Method::POST, "/setup/complete"),
        (Method::POST, "/logout"),
        (Method::PUT, "/admin/discord-settings"),
    ] {
        let response = app.form(&Browser::guest(), method, path, json!({})).await;
        response.assert_status(401);
        assert_json_eq(
            &json!({ "message": "Unauthenticated.", "redirect": "/eve" }),
            &response.json(),
        );
    }
}

#[tokio::test]
async fn expired_sessions_are_deleted() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.sign_in(user).await;
    app.clock.advance(Duration::days(401));
    app.sign_in(user).await;

    let deleted = bluebook::auth::session::delete_expired(&app.pool, app.clock.now(), 400)
        .await
        .expect("a cleanup");

    assert_eq!(deleted, 1);
    assert_eq!(app.count("sessions").await, 1);
}

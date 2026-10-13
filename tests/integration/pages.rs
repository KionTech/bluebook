//! tests/Feature/ExampleTest.php (the home page answers) and what every
//! page shares: the error answers, `/settings`, the health checks, the
//! stale Inertia tabs and the trailing slash.

use serde_json::json;

use crate::common::{APP_URL, Browser, TestApp, assert_json_eq};

#[tokio::test]
async fn returns_a_successful_response() {
    let app = TestApp::new().await;

    let response = app.page(&Browser::guest(), "/").await;

    response.assert_status(200);
    assert_json_eq(
        &json!({
            "name": "Bluebook",
            "auth": { "user": null },
            "sidebarOpen": true,
            "flash": { "success": null, "error": null, "info": null },
        }),
        &response.json(),
    );
}

#[tokio::test]
async fn the_sidebar_state_cookie_closes_the_sidebar() {
    let app = TestApp::new().await;

    let closed = app
        .get_with(
            &Browser::guest(),
            "/api/pages/home",
            &[("cookie", "sidebar_state=false")],
        )
        .await;
    assert_eq!(closed.json()["sidebarOpen"], false);
    let open = app
        .get_with(
            &Browser::guest(),
            "/api/pages/home",
            &[("cookie", "sidebar_state=true")],
        )
        .await;
    assert_eq!(open.json()["sidebarOpen"], true);
}

#[tokio::test]
async fn the_flash_message_shows_once() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    let browser = app.sign_in(user).await;

    app.put(
        &browser,
        &format!("/standings/{character}"),
        json!({ "should_sync": true }),
    )
    .await
    .assert_done();

    let first = app.page(&browser, "/dashboard").await.json();
    assert!(first["flash"]["success"].is_string());
    let second = app.page(&browser, "/dashboard").await.json();
    assert_eq!(second["flash"]["success"], json!(null));
}

#[tokio::test]
async fn settings_redirect_to_the_appearance_page() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    let browser = app.sign_in(user).await;

    for method in ["GET", "POST", "PUT", "PATCH", "DELETE"] {
        let response = app
            .form(
                &browser,
                method.parse().expect("a method"),
                "/settings",
                json!({}),
            )
            .await;
        response.assert_status(302);
        assert_eq!(response.location(), "/settings/appearance");
    }

    let guest = app.get(&Browser::guest(), "/settings").await;
    guest.assert_status(302);
    assert_eq!(guest.location(), format!("{APP_URL}/eve"));
}

#[tokio::test]
async fn unknown_paths_answer_the_not_found_page() {
    let app = TestApp::new().await;

    let response = app
        .get_with(&Browser::guest(), "/nope", &[("accept", "text/html")])
        .await;

    response.assert_status(404);
    assert!(response.text().contains("<title>Not Found</title>"));
    let json = app
        .get_with(&Browser::guest(), "/api/nope", &[("accept", "text/html")])
        .await;
    json.assert_status(404);
    assert_eq!(
        json.json()["message"],
        "The route api/nope could not be found."
    );
}

#[tokio::test]
async fn a_page_path_takes_no_other_method() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;

    let response = app
        .post(&app.sign_in(user).await, "/dashboard", json!({}))
        .await;

    response.assert_status(405);
    assert_eq!(
        response.json()["message"],
        "The POST method is not supported for route dashboard. Supported methods: GET, HEAD."
    );
}

#[tokio::test]
async fn the_health_checks_answer() {
    let app = TestApp::new().await;

    let page = app.get(&Browser::guest(), "/up").await;
    page.assert_status(200);
    assert!(page.text().contains("Application up"));
    let json = app
        .get_with(&Browser::guest(), "/up", &[("accept", "application/json")])
        .await;
    assert_json_eq(&json!({ "status": "up" }), &json.json());
    let health = app.get(&Browser::guest(), "/api/health").await;
    assert_json_eq(&json!({ "status": "ok", "database": "ok" }), &health.json());
}

#[tokio::test]
async fn the_local_disk_routes_are_forbidden() {
    let app = TestApp::new().await;

    for method in ["GET", "PUT"] {
        let response = app
            .request(
                method.parse().expect("a method"),
                "/storage/some/file.txt",
                crate::common::header_map(&[
                    ("host", crate::common::HOST),
                    ("accept", "text/html"),
                ]),
                axum::body::Body::empty(),
            )
            .await;
        response.assert_status(403);
        assert!(response.text().contains("<title>Forbidden</title>"));
    }
}

#[tokio::test]
async fn a_stale_inertia_tab_reloads_the_page() {
    let app = TestApp::new().await;

    let page = app
        .get_with(
            &Browser::guest(),
            "/dashboard?x=1",
            &[("x-inertia", "true"), ("x-forwarded-proto", "https")],
        )
        .await;
    page.assert_status(409);
    assert_eq!(
        page.header("x-inertia-location"),
        Some(format!("{APP_URL}/dashboard?x=1").as_str())
    );

    let form = app
        .request(
            axum::http::Method::PUT,
            "/standings/1",
            crate::common::header_map(&[
                ("host", crate::common::HOST),
                ("x-inertia", "true"),
                ("referer", &format!("{APP_URL}/dashboard")),
            ]),
            axum::body::Body::empty(),
        )
        .await;
    form.assert_status(409);
    assert_eq!(
        form.header("x-inertia-location"),
        Some(format!("{APP_URL}/dashboard").as_str())
    );
}

#[tokio::test]
async fn a_trailing_slash_reaches_the_same_form() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;

    app.put(
        &app.sign_in(user).await,
        &format!("/standings/{character}/"),
        json!({ "should_sync": true }),
    )
    .await
    .assert_done();

    assert!(app.should_sync(character).await);
}

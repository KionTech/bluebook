//! The harness itself and the route table: every route the api serves
//! answers, and every page SvelteKit renders has its props.

use axum::body::Body;
use axum::http::Method;
use serde_json::{Value, json};

use crate::common::{HOST, TestApp, header_map};

/// The page GETs SvelteKit renders (frontend/e2e/routes.spec.ts).
const PAGES: [&str; 7] = [
    "/",
    "/dashboard",
    "/settings/appearance",
    "/admin",
    "/admin/standing-requests",
    "/admin/pilots",
    "/admin/settings",
];

/// The routes the api answers itself.
const API_ROUTES: [(&str, &str); 22] = [
    ("PUT", "/admin/discord-settings"),
    ("PUT", "/admin/standing-requests/1"),
    ("PUT", "/admin/standings"),
    ("POST", "/admin/standings/sync"),
    ("PUT", "/auth/character/1"),
    ("DELETE", "/auth/character/1"),
    ("PUT", "/auth/character/1/main"),
    ("GET", "/eve"),
    ("GET", "/eve/callback"),
    ("POST", "/logout"),
    ("GET", "/settings"),
    ("POST", "/settings"),
    ("PUT", "/settings"),
    ("PATCH", "/settings"),
    ("DELETE", "/settings"),
    ("POST", "/setup/complete"),
    ("POST", "/standings/sync"),
    ("PUT", "/standings/1"),
    ("POST", "/standings/1/request"),
    ("GET", "/storage/file.txt"),
    ("PUT", "/storage/file.txt"),
    ("GET", "/up"),
];

#[tokio::test]
async fn every_api_route_exists() {
    let app = TestApp::new().await;
    for (method, path) in API_ROUTES {
        let response = app
            .request(
                method.parse::<Method>().expect("a method"),
                path,
                header_map(&[("host", HOST), ("accept", "application/json")]),
                Body::empty(),
            )
            .await;
        assert_ne!(response.status.as_u16(), 405, "{method} {path}");
        if response.status.as_u16() == 404
            && let Ok(body) = serde_json::from_slice::<Value>(&response.body)
        {
            let message = body["message"].as_str().unwrap_or_default();
            assert!(
                !message.contains("could not be found"),
                "{method} {path}: {message}"
            );
        }
    }
}

#[tokio::test]
async fn every_page_has_a_props_endpoint() {
    let app = TestApp::new().await;
    for page in PAGES {
        let response = app.page(&crate::common::Browser::guest(), page).await;
        assert_ne!(response.status.as_u16(), 404, "{page}");
    }
}

#[tokio::test]
async fn the_mocks_answer() {
    let app = TestApp::new().await;
    app.esi.affiliation(95_000_001, 98_000_001, None);
    let client = reqwest::Client::new();
    let answer: Value = client
        .post(format!("{}/characters/affiliation/", app.esi.url()))
        .json(&[95_000_001])
        .send()
        .await
        .expect("ESI answers")
        .json()
        .await
        .expect("JSON");
    assert_eq!(
        answer,
        json!([{ "character_id": 95_000_001, "corporation_id": 98_000_001 }])
    );
    app.esi.contacts(
        "characters/1",
        &[(10, "character", 5.0), (20, "alliance", -10.0)],
    );
    app.esi.page_size(1);
    let page = client
        .get(format!("{}/characters/1/contacts/?page=2", app.esi.url()))
        .send()
        .await
        .expect("ESI answers");
    assert_eq!(page.headers()["x-pages"], "2");
    let contacts: Value = page.json().await.expect("JSON");
    assert_eq!(contacts[0]["contact_id"], 20);
    let posted = client
        .post(app.discord.webhook())
        .json(&json!({ "content": "hello" }))
        .send()
        .await
        .expect("Discord answers");
    assert_eq!(posted.status().as_u16(), 204);
    assert_eq!(app.discord.requests()[0].json()["content"], "hello");
}

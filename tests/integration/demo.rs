//! The local demo login and its seed (FIXES.md, "The demo login").

use serde_json::json;

use crate::common::{APP_URL, Browser, TestApp};

#[tokio::test]
async fn the_demo_login_is_off_by_default() {
    let app = TestApp::new().await;

    app.get(&Browser::guest(), "/demo-login")
        .await
        .assert_status(404);
    let props = app.page(&Browser::guest(), "/").await.json();
    assert_eq!(props.get("demoLoginEnabled"), None);
    assert_eq!(app.count("users").await, 0);
}

#[tokio::test]
async fn the_demo_login_signs_in_the_demo_pilot() {
    let mut app = TestApp::new().await;
    app.reconfigure(|config| config.demo.enabled = true);

    let response = app.get(&Browser::guest(), "/demo-login").await;

    response.assert_status(302);
    assert_eq!(response.location(), format!("{APP_URL}/dashboard"));
    let browser = response.browser(&Browser::guest());
    let props = app.page(&browser, "/dashboard").await;
    props.assert_status(200);
    let props = props.json();
    assert_eq!(props["auth"]["user"]["name"], "Demo Pilot");
    assert_eq!(props["auth"]["active_character"]["id"], 2_100_000_001);
    assert_eq!(
        app.page(&Browser::guest(), "/").await.json()["demoLoginEnabled"],
        json!(true)
    );
}

#[tokio::test]
async fn the_demo_login_signs_in_as_a_given_user() {
    let mut app = TestApp::new().await;
    app.reconfigure(|config| config.demo.enabled = true);
    let user = app.user().await;
    app.character_of(user).await;

    let response = app
        .get(&Browser::guest(), &format!("/demo-login?user={user}"))
        .await;

    response.assert_status(302);
    let browser = response.browser(&Browser::guest());
    let props = app.page(&browser, "/dashboard").await.json();
    assert_eq!(props["auth"]["user"]["id"], user);
}

#[tokio::test]
async fn the_demo_seed_builds_a_world_and_can_run_again() {
    let mut app = TestApp::new().await;
    app.reconfigure(|config| {
        config.demo.enabled = true;
        config.admin_character_ids = vec![2_100_000_001];
    });

    for _ in 0..2 {
        bluebook::demo::seed(&app.pool, app.clock.now(), 2_100_000_001)
            .await
            .expect("the demo seed");
    }

    let browser = app
        .get(&Browser::guest(), "/demo-login")
        .await
        .browser(&Browser::guest());
    let dashboard = app.page(&browser, "/dashboard").await.json();
    assert_eq!(dashboard["auth"]["is_admin"], true);
    // The login leaves the seeded pilot as it is: in the source alliance.
    assert_eq!(dashboard["characters"][0]["inherits_source"], true);
    assert_eq!(
        dashboard["auth"]["active_character"]["corporation"]["name"],
        "Demo Corp"
    );
    assert!(
        dashboard["standings"]
            .as_array()
            .is_some_and(|standings| standings.len() >= 5)
    );
    let admin = app.page(&browser, "/admin/standing-requests").await.json();
    assert!(
        admin["standingRequests"]["total"]
            .as_i64()
            .is_some_and(|total| total >= 2)
    );
    assert_eq!(app.count("standings_sources").await, 1);
}

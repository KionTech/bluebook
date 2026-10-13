//! The admin console: the overview, resolving requests, the Discord
//! settings, the requests list and the pilots behind the admin gate.

use serde_json::json;

use crate::common::factories::{NewCharacter, NewRequest, Owner};
use crate::common::{Browser, TestApp, assert_json_eq};

/// A user owning the configured admin character.
async fn admin(app: &mut TestApp) -> Browser {
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.admins(&[character]);
    app.sign_in(user).await
}

/// A user whose only character is not the admin (999999 is).
async fn pilot(app: &mut TestApp) -> Browser {
    let user = app.user().await;
    app.character_of(user).await;
    app.admins(&[999_999]);
    app.sign_in(user).await
}

#[tokio::test]
async fn shows_the_overview_with_stats_and_recent_requests_to_the_admin() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    let requester = app
        .character(NewCharacter {
            name: Some("Requester"),
            ..NewCharacter::default()
        })
        .await;
    app.standing_request(NewRequest {
        requested_by: Some(requester),
        status: Some("pending"),
        ..NewRequest::default()
    })
    .await;

    let response = app.page(&browser, "/admin").await;
    response.assert_status(200);
    let props = response.json();
    assert_eq!(props["stats"]["pending_requests"], 1);
    assert_eq!(props["stats"]["pilots"], 2);
    assert_eq!(props["recentRequests"].as_array().map(Vec::len), Some(1));
}

#[tokio::test]
async fn treats_any_of_the_comma_separated_admin_characters_as_an_admin() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.admins(&[111_111, character, 222_222]);

    app.page(&app.sign_in(user).await, "/admin")
        .await
        .assert_status(200);
}

#[tokio::test]
async fn forbids_non_admins_from_the_administration_page() {
    let mut app = TestApp::new().await;
    let browser = pilot(&mut app).await;

    let response = app.page(&browser, "/admin").await;
    response.assert_status(403);
    assert_eq!(response.json()["message"], "This action is unauthorized.");
}

#[tokio::test]
async fn lets_the_admin_mark_a_request_as_done() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    let request = app
        .standing_request(NewRequest {
            status: Some("pending"),
            ..NewRequest::default()
        })
        .await;

    let redirect = app
        .put(
            &browser,
            &format!("/admin/standing-requests/{request}"),
            json!({ "status": "done" }),
        )
        .await
        .assert_done();

    assert_eq!(redirect, None);
    assert_eq!(app.request_status(request).await, "done");
}

#[tokio::test]
async fn lets_the_admin_reject_a_request() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    let request = app
        .standing_request(NewRequest {
            status: Some("pending"),
            ..NewRequest::default()
        })
        .await;

    app.put(
        &browser,
        &format!("/admin/standing-requests/{request}"),
        json!({ "status": "rejected" }),
    )
    .await
    .assert_done();

    assert_eq!(app.request_status(request).await, "rejected");
}

#[tokio::test]
async fn forbids_non_admins_from_resolving_requests() {
    let mut app = TestApp::new().await;
    let browser = pilot(&mut app).await;
    let request = app
        .standing_request(NewRequest {
            status: Some("pending"),
            ..NewRequest::default()
        })
        .await;

    app.put(
        &browser,
        &format!("/admin/standing-requests/{request}"),
        json!({ "status": "done" }),
    )
    .await
    .assert_status(403);

    assert_eq!(app.request_status(request).await, "pending");
}

#[tokio::test]
async fn lets_the_admin_update_the_discord_settings() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;

    app.put(
        &browser,
        "/admin/discord-settings",
        json!({ "webhook_url": "https://discord.com/api/webhooks/1/abc", "role_id": "123456" }),
    )
    .await
    .assert_done();

    assert_json_eq(
        &json!([{ "webhook_url": "https://discord.com/api/webhooks/1/abc", "role_id": "123456" }]),
        &json!(
            app.rows("select webhook_url, role_id from discord_settings")
                .await
        ),
    );
}

#[tokio::test]
async fn validates_the_discord_settings() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;

    let errors = app
        .put(
            &browser,
            "/admin/discord-settings",
            json!({ "webhook_url": "not-a-url", "role_id": "abc" }),
        )
        .await
        .assert_invalid();

    assert_json_eq(
        &json!({
            "webhook_url": "The webhook url field must be a valid URL.",
            "role_id": "The role id field format is invalid.",
        }),
        &errors,
    );
}

#[tokio::test]
async fn forbids_non_admins_from_updating_discord_settings() {
    let mut app = TestApp::new().await;
    let browser = pilot(&mut app).await;

    app.put(
        &browser,
        "/admin/discord-settings",
        json!({ "webhook_url": "https://discord.com/x" }),
    )
    .await
    .assert_status(403);
    assert_eq!(app.count("discord_settings").await, 0);
}

#[tokio::test]
async fn lists_every_registered_character_under_its_account_on_the_pilots_page() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    let pilot = app.user().await;
    let alt = app
        .character(NewCharacter {
            name: Some("Alt Pilot"),
            owner: Owner::User(pilot),
            ..NewCharacter::default()
        })
        .await;
    let main = app
        .character(NewCharacter {
            name: Some("Main Pilot"),
            owner: Owner::User(pilot),
            ..NewCharacter::default()
        })
        .await;
    app.set_main(pilot, main).await;

    let response = app.page(&browser, "/admin/pilots").await;
    response.assert_status(200);
    let props = response.json();
    let rows = props["characters"].as_array().expect("characters");
    let row = |id: i64| {
        rows.iter()
            .find(|row| row["id"] == id)
            .expect("a row")
            .clone()
    };
    assert_eq!(row(main)["is_main"], true);
    assert_eq!(row(alt)["is_main"], false);
    assert_eq!(row(main)["account"]["name"], "Main Pilot");
    assert_eq!(row(alt)["account"]["name"], "Main Pilot");
}

#[tokio::test]
async fn shows_pending_requests_on_the_standing_requests_page_by_default() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    let requester = app
        .character(NewCharacter {
            name: Some("Requester"),
            ..NewCharacter::default()
        })
        .await;
    app.standing_request(NewRequest {
        requested_by: Some(requester),
        ..NewRequest::default()
    })
    .await;
    app.standing_request(NewRequest {
        status: Some("done"),
        ..NewRequest::default()
    })
    .await;

    let response = app.page(&browser, "/admin/standing-requests").await;
    response.assert_status(200);
    let props = response.json();
    assert_eq!(
        props["standingRequests"]["data"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(props["standingRequests"]["data"][0]["status"], "pending");
    assert_eq!(props["counts"]["pending"], 1);
    assert_eq!(props["counts"]["done"], 1);
    assert_eq!(props["filters"]["status"], "pending");
}

#[tokio::test]
async fn shows_done_requests_only_when_the_done_filter_is_selected() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    app.standing_request(NewRequest {
        status: Some("pending"),
        ..NewRequest::default()
    })
    .await;
    app.standing_request(NewRequest {
        status: Some("done"),
        ..NewRequest::default()
    })
    .await;

    let props = app
        .page(&browser, "/admin/standing-requests?status=done")
        .await
        .json();
    assert_eq!(
        props["standingRequests"]["data"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(props["standingRequests"]["data"][0]["status"], "done");
    assert_eq!(props["filters"]["status"], "done");
}

#[tokio::test]
async fn paginates_the_standing_requests_list() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    for _ in 0..30 {
        app.standing_request(NewRequest {
            status: Some("pending"),
            ..NewRequest::default()
        })
        .await;
    }

    let props = app.page(&browser, "/admin/standing-requests").await.json();
    assert_eq!(
        props["standingRequests"]["data"].as_array().map(Vec::len),
        Some(25)
    );
    assert_eq!(props["standingRequests"]["total"], 30);
}

#[tokio::test]
async fn passes_the_inherited_effective_standing_for_each_request_to_the_admin_console() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    app.corporation(4000, Some("Blue Corp"), None, None).await;
    let admin = app
        .character(NewCharacter {
            owner: Owner::User(user),
            corporation_id: Some(4000),
            ..NewCharacter::default()
        })
        .await;
    app.admins(&[admin]);
    app.source("character", 999).await;
    app.source_contact(4000, "corporation", 5.0).await;
    app.standing_request(NewRequest {
        subject_type: Some("character"),
        subject_id: Some(admin),
        requested_by: Some(admin),
        status: Some("pending"),
        ..NewRequest::default()
    })
    .await;

    let props = app
        .page(&app.sign_in(user).await, "/admin/standing-requests")
        .await
        .json();
    assert_json_eq(
        &json!({
            "standing": 5,
            "source": "corporation",
            "via_type": "corporation",
            "via_id": 4000,
            "via_name": "Blue Corp",
        }),
        &props["standingRequests"]["data"][0]["effective_standing"],
    );
}

#[tokio::test]
async fn names_a_request_by_its_subject_after_the_requester_moved() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    app.corporation(4001, Some("Asked Corp"), None, None).await;
    app.corporation(4002, Some("New Corp"), None, None).await;
    app.alliance(5001, Some("Asked Alliance"), None).await;
    let requester = app
        .character(NewCharacter {
            name: Some("Mover"),
            corporation_id: Some(4002),
            ..NewCharacter::default()
        })
        .await;
    for (kind, id) in [
        ("corporation", 4001),
        ("alliance", 5001),
        ("corporation", 4003),
    ] {
        app.standing_request(NewRequest {
            subject_type: Some(kind),
            subject_id: Some(id),
            requested_by: Some(requester),
            ..NewRequest::default()
        })
        .await;
    }

    let props = app.page(&browser, "/admin/standing-requests").await.json();
    let mut names: Vec<(i64, serde_json::Value)> = props["standingRequests"]["data"]
        .as_array()
        .expect("requests")
        .iter()
        .map(|request| {
            (
                request["subject"]["id"].as_i64().expect("an id"),
                request["subject"]["name"].clone(),
            )
        })
        .collect();
    names.sort_by_key(|(id, _)| *id);
    assert_eq!(
        names,
        vec![
            (4001, json!("Asked Corp")),
            (4003, json!(null)),
            (5001, json!("Asked Alliance")),
        ]
    );
}

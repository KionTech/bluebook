//! tests/Feature/StandingRequests/StandingRequestTest.php (asking for a
//! standing, the Discord announcement) and EffectiveStandingResolverTest.php
//! (the standing a request's subject already has, as the admin's request
//! list shows it).

use serde_json::{Value, json};

use bluebook::jobs::Job;

use crate::common::factories::{NewCharacter, NewRequest, Owner};
use crate::common::{APP_URL, TestApp, assert_json_eq};

async fn requests(app: &TestApp) -> Value {
    json!(app.rows(
        "select subject_type, subject_id, requested_by_character_id, status from standing_requests order by id"
    ).await)
}

#[tokio::test]
async fn lets_a_user_request_standing_for_their_character() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.source("character", 999).await;

    app.post(
        &app.sign_in(user).await,
        &format!("/standings/{character}/request"),
        json!({ "type": "character" }),
    )
    .await
    .assert_done();

    assert_json_eq(
        &json!([{
            "subject_type": "character",
            "subject_id": character,
            "requested_by_character_id": character,
            "status": "pending",
        }]),
        &requests(&app).await,
    );
    assert_json_eq(
        &json!([{ "standing_request_id": 1 }]),
        &json!(app.jobs_of("discord_request").await),
    );
}

#[tokio::test]
async fn lets_a_user_request_standing_for_their_corporation() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.corporation(2000, Some("My Corp"), None, None).await;
    let character = app
        .character(NewCharacter {
            owner: Owner::User(user),
            corporation_id: Some(2000),
            ..NewCharacter::default()
        })
        .await;
    app.source("character", 999).await;

    app.post(
        &app.sign_in(user).await,
        &format!("/standings/{character}/request"),
        json!({ "type": "corporation" }),
    )
    .await
    .assert_done();

    assert_json_eq(
        &json!([{
            "subject_type": "corporation",
            "subject_id": 2000,
            "requested_by_character_id": character,
            "status": "pending",
        }]),
        &requests(&app).await,
    );
}

#[tokio::test]
async fn does_not_allow_a_request_when_the_subject_already_has_standing() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.source("character", 999).await;
    app.source_contact(character, "character", 5.0).await;

    app.post(
        &app.sign_in(user).await,
        &format!("/standings/{character}/request"),
        json!({ "type": "character" }),
    )
    .await
    .assert_done();

    assert_json_eq(&json!([]), &requests(&app).await);
}

#[tokio::test]
async fn forbids_requesting_standing_for_another_users_character() {
    let app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    let other = app.character(NewCharacter::default()).await;
    app.source("character", 999).await;

    app.post(
        &app.sign_in(user).await,
        &format!("/standings/{other}/request"),
        json!({ "type": "character" }),
    )
    .await
    .assert_status(403);
}

#[tokio::test]
async fn does_not_duplicate_a_pending_request_for_the_same_subject() {
    let app = TestApp::new().await;
    let user = app.user().await;
    let character = app.character_of(user).await;
    app.source("character", 999).await;
    app.standing_request(NewRequest {
        subject_type: Some("character"),
        subject_id: Some(character),
        requested_by: Some(character),
        status: Some("pending"),
        ..NewRequest::default()
    })
    .await;

    app.post(
        &app.sign_in(user).await,
        &format!("/standings/{character}/request"),
        json!({ "type": "character" }),
    )
    .await
    .assert_done();

    assert_eq!(app.jobs().await.len(), 0);
    assert_eq!(app.count("standing_requests").await, 1);
}

#[tokio::test]
async fn posts_a_standing_request_to_the_configured_discord_webhook_and_pings_the_role() {
    let app = TestApp::new().await;
    app.discord_settings(Some(&app.discord.webhook()), Some("123456789"))
        .await;
    let character = app
        .character(NewCharacter {
            name: Some("Lonewolf"),
            ..NewCharacter::default()
        })
        .await;
    let request = app
        .standing_request(NewRequest {
            subject_type: Some("character"),
            subject_id: Some(character),
            requested_by: Some(character),
            ..NewRequest::default()
        })
        .await;

    bluebook::jobs::enqueue(
        &app.state,
        &Job::DiscordRequest {
            standing_request_id: request,
        },
    )
    .await
    .expect("a queued announcement");
    bluebook::jobs::run_pending(&app.state).await;

    let requests = app.discord.requests();
    assert_eq!(requests.len(), 1);
    assert_json_eq(
        &json!({
            "content": "<@&123456789> A character requested standings.",
            "embeds": [{
                "title": "Lonewolf",
                "description": "Requested by Lonewolf. Review it in the administration page.\n```\nLonewolf\n```",
                "url": format!("{APP_URL}/admin"),
                "thumbnail": {
                    "url": format!("https://images.evetech.net/characters/{character}/portrait?size=128"),
                },
                "fields": [
                    { "name": "Type", "value": "Character", "inline": true },
                    { "name": "Requested by", "value": "Lonewolf", "inline": true },
                ],
            }],
            "allowed_mentions": { "roles": ["123456789"] },
        }),
        &requests[0].json(),
    );
}

#[tokio::test]
async fn skips_the_discord_webhook_when_none_is_configured() {
    let app = TestApp::new().await;
    let request = app.standing_request(NewRequest::default()).await;

    bluebook::jobs::enqueue(
        &app.state,
        &Job::DiscordRequest {
            standing_request_id: request,
        },
    )
    .await
    .expect("a queued announcement");
    bluebook::jobs::run_pending(&app.state).await;

    assert_eq!(app.discord.requests().len(), 0);
}

#[tokio::test]
async fn falls_back_to_the_webhook_of_the_environment() {
    let mut app = TestApp::new().await;
    let webhook = app.discord.webhook();
    app.reconfigure(|config| config.discord_webhook = Some(webhook));
    let request = app.standing_request(NewRequest::default()).await;

    bluebook::jobs::enqueue(
        &app.state,
        &Job::DiscordRequest {
            standing_request_id: request,
        },
    )
    .await
    .expect("a queued announcement");
    bluebook::jobs::run_pending(&app.state).await;

    let requests = app.discord.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].json()["allowed_mentions"], json!(null));
}

// EffectiveStandingResolverTest.php, through the admin's request list.

async fn effective_standing(app: &mut TestApp, character: i64) -> Value {
    app.standing_request(NewRequest {
        subject_type: Some("character"),
        subject_id: Some(character),
        requested_by: Some(character),
        ..NewRequest::default()
    })
    .await;
    let user = app.user().await;
    let admin = app.character_of(user).await;
    app.admins(&[admin]);
    let props = app
        .page(&app.sign_in(user).await, "/admin/standing-requests")
        .await
        .json();
    props["standingRequests"]["data"][0]["effective_standing"].clone()
}

#[tokio::test]
async fn resolves_a_standing_set_directly_on_the_entity() {
    let mut app = TestApp::new().await;
    let character = app.character(NewCharacter::default()).await;
    app.source_contact(character, "character", 7.0).await;

    assert_json_eq(
        &json!({
            "standing": 7,
            "source": "direct",
            "via_type": "character",
            "via_id": character,
            "via_name": app.scalar(&format!("select name from characters where id = {character}")).await,
        }),
        &effective_standing(&mut app, character).await,
    );
}

#[tokio::test]
async fn inherits_a_standing_from_the_requesting_characters_corporation() {
    let mut app = TestApp::new().await;
    app.corporation(4000, Some("Blue Corp"), None, None).await;
    let character = app
        .character(NewCharacter {
            corporation_id: Some(4000),
            ..NewCharacter::default()
        })
        .await;
    app.source_contact(4000, "corporation", 5.0).await;

    assert_json_eq(
        &json!({
            "standing": 5,
            "source": "corporation",
            "via_type": "corporation",
            "via_id": 4000,
            "via_name": "Blue Corp",
        }),
        &effective_standing(&mut app, character).await,
    );
}

#[tokio::test]
async fn prefers_a_direct_standing_over_an_inherited_one() {
    let mut app = TestApp::new().await;
    app.corporation(4000, Some("Blue Corp"), None, None).await;
    let character = app
        .character(NewCharacter {
            corporation_id: Some(4000),
            ..NewCharacter::default()
        })
        .await;
    app.source_contact(character, "character", 3.0).await;
    app.source_contact(4000, "corporation", 10.0).await;

    let standing = effective_standing(&mut app, character).await;
    assert_eq!(standing["standing"], 3);
    assert_eq!(standing["source"], "direct");
}

#[tokio::test]
async fn returns_null_when_neither_the_entity_nor_a_parent_has_a_standing() {
    let mut app = TestApp::new().await;
    let character = app.character(NewCharacter::default()).await;

    assert_eq!(effective_standing(&mut app, character).await, json!(null));
}

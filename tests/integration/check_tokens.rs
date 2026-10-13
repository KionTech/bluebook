//! tests/Feature/Console/CheckSyncTokensTest.php: the daily mail to
//! synced characters whose write token lapsed, and the mail itself.

use serde_json::json;

use bluebook::jobs::Job;

use crate::common::factories::{NewCharacter, Owner};
use crate::common::{
    APP_URL, READ_CORPORATION_CONTACTS, SEND_MAIL, TestApp, WRITE_CHARACTER_CONTACTS,
    assert_json_eq,
};

/// `tokenCheckSetup()`: a corporation source whose sender can read it and
/// send mail.
async fn setup(app: &mut TestApp) -> i64 {
    app.corporation(2000, Some("Source Corp"), None, None).await;
    let sender = app
        .character(NewCharacter {
            corporation_id: Some(2000),
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(sender, &[READ_CORPORATION_CONTACTS, SEND_MAIL])
        .await;
    app.admins(&[sender]);
    app.source("corporation", 2000).await;
    sender
}

async fn synced(app: &TestApp, name: &str, corporation: Option<i64>) -> i64 {
    app.character(NewCharacter {
        name: Some(name),
        should_sync: true,
        corporation_id: corporation,
        ..NewCharacter::default()
    })
    .await
}

#[tokio::test]
async fn mails_synced_characters_whose_write_contacts_token_expired() {
    let mut app = TestApp::new().await;
    let sender = setup(&mut app).await;
    let lapsed = synced(&app, "Lapsed", None).await;

    assert_eq!(bluebook::standings::tokens::check(&app.state).await, 0);

    assert_json_eq(
        &json!([{ "sender_id": sender, "recipient_id": lapsed }]),
        &json!(app.jobs_of("token_mail").await),
    );
}

#[tokio::test]
async fn does_not_mail_characters_that_still_have_a_token() {
    let mut app = TestApp::new().await;
    setup(&mut app).await;
    let ok = synced(&app, "Fine", None).await;
    app.grant_scopes(ok, &[WRITE_CHARACTER_CONTACTS]).await;

    assert_eq!(bluebook::standings::tokens::check(&app.state).await, 0);

    assert_eq!(app.jobs_of("token_mail").await.len(), 0);
}

#[tokio::test]
async fn does_not_mail_characters_that_inherit_the_source() {
    let mut app = TestApp::new().await;
    setup(&mut app).await;
    synced(&app, "Member", Some(2000)).await;

    assert_eq!(bluebook::standings::tokens::check(&app.state).await, 0);

    assert_eq!(app.jobs_of("token_mail").await.len(), 0);
}

#[tokio::test]
async fn mails_each_lapsed_character_at_most_once() {
    let mut app = TestApp::new().await;
    setup(&mut app).await;
    synced(&app, "Lapsed", None).await;

    bluebook::standings::tokens::check(&app.state).await;
    bluebook::standings::tokens::check(&app.state).await;

    assert_eq!(app.jobs_of("token_mail").await.len(), 1);
}

#[tokio::test]
async fn mails_a_lapsed_character_again_after_three_days() {
    let mut app = TestApp::new().await;
    setup(&mut app).await;
    synced(&app, "Lapsed", None).await;

    bluebook::standings::tokens::check(&app.state).await;
    app.clock
        .advance(time::Duration::days(3) - time::Duration::seconds(1));
    bluebook::standings::tokens::check(&app.state).await;
    assert_eq!(app.jobs_of("token_mail").await.len(), 1);
    app.clock.advance(time::Duration::seconds(1));
    bluebook::standings::tokens::check(&app.state).await;
    assert_eq!(app.jobs_of("token_mail").await.len(), 2);
}

#[tokio::test]
async fn does_nothing_when_the_source_character_cannot_send_mail() {
    let mut app = TestApp::new().await;
    app.corporation(2000, Some("Source Corp"), None, None).await;
    let sender = app
        .character(NewCharacter {
            corporation_id: Some(2000),
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(sender, &[READ_CORPORATION_CONTACTS]).await;
    app.admins(&[sender]);
    app.source("corporation", 2000).await;
    synced(&app, "Lapsed", None).await;

    assert_eq!(bluebook::standings::tokens::check(&app.state).await, 0);

    assert_eq!(app.jobs_of("token_mail").await.len(), 0);
}

#[tokio::test]
async fn sends_an_eve_mail_from_the_source_character_to_the_lapsed_character() {
    let app = TestApp::new().await;
    let sender = app.character(NewCharacter::default()).await;
    app.grant_scopes(sender, &[SEND_MAIL]).await;
    let recipient = app
        .character(NewCharacter {
            name: Some("Lapsed Pilot"),
            ..NewCharacter::default()
        })
        .await;

    bluebook::jobs::enqueue(
        &app.state,
        &Job::TokenMail {
            sender_id: sender,
            recipient_id: recipient,
        },
    )
    .await
    .expect("a queued mail");
    bluebook::jobs::run_pending(&app.state).await;

    let requests = app.esi.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].line(),
        format!("POST /characters/{sender}/mail/")
    );
    assert_json_eq(
        &json!({
            "approved_cost": 0,
            "body": format!(
                "Hello Lapsed Pilot,\n\nYour contact permissions for Bluebook have lapsed, so your standings can't be synced. Please grant them again here:\n{APP_URL}/eve?add_to_account=1&scopes=esi-characters.read_contacts.v1%2Cesi-characters.write_contacts.v1\n\nso the service won't get interrupted."
            ),
            "recipients": [{ "recipient_id": recipient, "recipient_type": "character" }],
            "subject": "Bluebook \u{2014} re-authentication required",
        }),
        &requests[0].json(),
    );
}

/// FIXES.md, "Removed characters were mailed about lapsed tokens forever".
#[tokio::test]
async fn does_not_mail_characters_without_an_account() {
    let mut app = TestApp::new().await;
    setup(&mut app).await;
    app.character(NewCharacter {
        name: Some("Removed"),
        owner: Owner::Nobody,
        should_sync: true,
        ..NewCharacter::default()
    })
    .await;

    assert_eq!(bluebook::standings::tokens::check(&app.state).await, 0);

    assert_eq!(app.jobs_of("token_mail").await.len(), 0);
}

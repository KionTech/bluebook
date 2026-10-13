//! tests/Feature/Console/SyncStandingsCommandTest.php: the five-minute
//! `standings:sync`, its fan-out to the characters, the downtime skip and
//! the Discord alert for an unreadable source.

use serde_json::json;
use time::macros::datetime;

use bluebook::jobs::Job;

use crate::common::factories::NewCharacter;
use crate::common::{
    APP_URL, READ_CHARACTER_CONTACTS, READ_CORPORATION_CONTACTS, TestApp, WRITE_CHARACTER_CONTACTS,
    assert_json_eq,
};

/// A corporation source with an admin that can read it.
async fn readable_source(app: &mut TestApp) {
    app.corporation(2000, Some("Source Corp"), None, None).await;
    let admin = app
        .character(NewCharacter {
            corporation_id: Some(2000),
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(admin, &[READ_CORPORATION_CONTACTS]).await;
    app.admins(&[admin]);
    app.source("corporation", 2000).await;
}

async fn syncable(app: &TestApp) -> i64 {
    let character = app
        .character(NewCharacter {
            should_sync: true,
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(character, &[WRITE_CHARACTER_CONTACTS])
        .await;
    character
}

/// The source answers one character contact, as the PHP tests faked it.
fn one_contact(app: &TestApp) {
    app.esi
        .contacts("corporations/2000", &[(100, "character", 5.0)]);
    app.esi.name(100, "character", "Contact One");
    app.esi.affiliation(100, 2001, None);
    app.esi.corporation(2001, "Contact Corp", None);
}

#[tokio::test]
async fn refreshes_the_source_and_queues_every_opted_in_character() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    let syncable = syncable(&app).await;
    // Opted out / missing scope characters must be ignored.
    app.character(NewCharacter::default()).await;
    let no_scope = app
        .character(NewCharacter {
            should_sync: true,
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(no_scope, &[READ_CHARACTER_CONTACTS]).await;
    // A character inside the source corporation already inherits and must
    // be skipped.
    let inheriting = app
        .character(NewCharacter {
            should_sync: true,
            corporation_id: Some(2000),
            ..NewCharacter::default()
        })
        .await;
    app.grant_scopes(inheriting, &[WRITE_CHARACTER_CONTACTS])
        .await;
    one_contact(&app);

    assert_eq!(bluebook::standings::sync::run(&app.state, false).await, 0);

    assert_json_eq(
        &json!([{ "character_id": syncable }]),
        &json!(app.jobs_of("character_sync").await),
    );
}

#[tokio::test]
async fn does_not_queue_any_syncs_when_the_source_is_unchanged() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.source_contact(100, "character", 5.0).await;
    syncable(&app).await;
    app.esi
        .contacts("corporations/2000", &[(100, "character", 5.0)]);

    assert_eq!(bluebook::standings::sync::run(&app.state, false).await, 0);

    assert_eq!(app.jobs().await.len(), 0);
}

#[tokio::test]
async fn queues_syncs_for_unchanged_standings_when_forced() {
    let mut app = TestApp::new().await;
    readable_source(&mut app).await;
    app.source_contact(100, "character", 5.0).await;
    syncable(&app).await;
    app.esi
        .contacts("corporations/2000", &[(100, "character", 5.0)]);

    assert_eq!(bluebook::standings::sync::run(&app.state, true).await, 0);

    assert_eq!(app.jobs_of("character_sync").await.len(), 1);
}

#[tokio::test]
async fn skips_the_sync_during_eve_downtime_without_calling_esi_or_alerting() {
    let mut app = TestApp::at(datetime!(2026-06-25 11:05:00 UTC)).await;
    // A source that cannot be read would normally fail and alert Discord.
    app.source("corporation", 2000).await;
    app.admins(&[]);

    assert_eq!(bluebook::standings::sync::run(&app.state, false).await, 0);

    assert_eq!(app.esi.requests().len(), 0);
    assert_eq!(app.jobs().await.len(), 0);
}

#[tokio::test]
async fn fails_when_no_source_is_configured() {
    let app = TestApp::new().await;

    assert_eq!(bluebook::standings::sync::run(&app.state, false).await, 1);

    assert_eq!(app.jobs().await.len(), 0);
}

#[tokio::test]
async fn alerts_discord_at_most_once_per_hour_when_the_source_cannot_be_read() {
    let mut app = TestApp::new().await;
    // Source configured, but no admin character can read it.
    app.source("corporation", 2000).await;
    app.admins(&[]);

    assert_eq!(bluebook::standings::sync::run(&app.state, false).await, 1);
    assert_eq!(app.jobs_of("discord_unreadable").await.len(), 1);

    // Throttled: a second run within the same hour does not re-alert.
    assert_eq!(bluebook::standings::sync::run(&app.state, false).await, 1);
    assert_eq!(app.jobs_of("discord_unreadable").await.len(), 1);

    // The next clock hour alerts again.
    app.clock.set(datetime!(2026-06-25 15:00:00 UTC));
    assert_eq!(bluebook::standings::sync::run(&app.state, false).await, 1);
    assert_eq!(app.jobs_of("discord_unreadable").await.len(), 2);
}

#[tokio::test]
async fn does_not_alert_discord_when_no_source_is_configured() {
    let app = TestApp::new().await;

    assert_eq!(bluebook::standings::sync::run(&app.state, false).await, 1);

    assert_eq!(app.jobs_of("discord_unreadable").await.len(), 0);
}

#[tokio::test]
async fn posts_a_source_unreadable_alert_to_the_discord_webhook() {
    let app = TestApp::new().await;
    app.discord_settings(Some(&app.discord.webhook()), Some("777"))
        .await;
    app.corporation(2000, Some("Source Corp"), None, None).await;
    app.source("corporation", 2000).await;

    bluebook::jobs::enqueue(&app.state, &Job::DiscordUnreadable)
        .await
        .expect("a queued alert");
    bluebook::jobs::run_pending(&app.state).await;

    let requests = app.discord.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/webhook");
    let body = requests[0].text();
    assert!(body.contains("Source Corp"), "{body}");
    assert!(body.contains("<@&777>"), "{body}");
    assert!(body.contains("Click here to re-authenticate"), "{body}");
}

/// FIXES.md, "The "source unreadable" alert linked a login without
/// scopes".
#[tokio::test]
async fn the_unreadable_alert_links_the_admin_grant() {
    let app = TestApp::new().await;
    app.discord_settings(Some(&app.discord.webhook()), None)
        .await;
    app.source("alliance", 3000).await;

    bluebook::jobs::enqueue(&app.state, &Job::DiscordUnreadable)
        .await
        .expect("a queued alert");
    bluebook::jobs::run_pending(&app.state).await;

    let grant = format!(
        "{APP_URL}/eve?add_to_account=1&scopes=publicData%2Cesi-characters.read_contacts.v1%2Cesi-characters.write_contacts.v1%2Cesi-corporations.read_contacts.v1%2Cesi-alliances.read_contacts.v1%2Cesi-mail.send_mail.v1"
    );
    assert_json_eq(
        &json!({
            "content": ":warning: The standings source could not be read.",
            "embeds": [{
                "title": "Standings source unreadable",
                "description": format!(
                    "Couldn't read **3000**'s contacts. An admin character's ESI token may be missing or lacking permissions.\n\n**[Click here to re-authenticate]({grant})**"
                ),
                "url": grant,
            }],
        }),
        &app.discord.requests()[0].json(),
    );
}

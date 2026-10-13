//! The admin's pilot list, flat and grouped by corporation or alliance,
//! with effective standings.

use serde_json::json;

use crate::common::factories::{NewCharacter, Owner};
use crate::common::{Browser, TestApp, assert_json_eq};

/// The admin is "Admin Pilot", so alphabetical
/// assertions hold.
async fn admin(app: &mut TestApp) -> Browser {
    let user = app.user().await;
    let character = app
        .character(NewCharacter {
            name: Some("Admin Pilot"),
            owner: Owner::User(user),
            ..NewCharacter::default()
        })
        .await;
    app.admins(&[character]);
    app.sign_in(user).await
}

async fn character(app: &TestApp, name: &str, owner: Owner, corporation: Option<i64>) -> i64 {
    app.character(NewCharacter {
        name: Some(name),
        owner,
        corporation_id: corporation,
        ..NewCharacter::default()
    })
    .await
}

#[tokio::test]
async fn lists_every_registered_character_with_its_account_and_standing() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    app.corporation(5000, Some("Blue Corp"), Some("BLUE"), None)
        .await;
    app.source_contact(5000, "corporation", 5.0).await;
    let pilot = app.user().await;
    let main = character(&app, "Main Pilot", Owner::User(pilot), Some(5000)).await;
    character(&app, "Alt Pilot", Owner::User(pilot), None).await;
    app.set_main(pilot, main).await;
    // Characters without an account never show.
    character(&app, "Unregistered", Owner::Nobody, None).await;

    let response = app.page(&browser, "/admin/pilots").await;
    response.assert_status(200);
    let props = response.json();
    assert_eq!(props["groups"], json!(null));
    let rows = props["characters"].as_array().expect("characters");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[1]["name"], "Alt Pilot");
    assert_eq!(rows[1]["account"]["name"], "Main Pilot");
    assert_eq!(rows[1]["standing"], json!(null));
    assert_eq!(rows[2]["name"], "Main Pilot");
    assert_eq!(rows[2]["is_main"], true);
    assert_json_eq(
        &json!({ "value": 5, "source": "corporation" }),
        &rows[2]["standing"],
    );
}

#[tokio::test]
async fn passes_the_search_term_through_for_client_side_filtering() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    let other = app.user().await;
    character(&app, "Jita Trader", Owner::User(other), None).await;

    // The characters list stays complete; the client applies the filter.
    let props = app.page(&browser, "/admin/pilots?search=Jita").await.json();
    assert_eq!(props["characters"].as_array().map(Vec::len), Some(2));
    assert_eq!(props["filters"]["search"], "Jita");
}

#[tokio::test]
async fn forbids_non_admins_from_the_pilots_page() {
    let mut app = TestApp::new().await;
    let user = app.user().await;
    app.character_of(user).await;
    app.admins(&[999_999]);

    app.page(&app.sign_in(user).await, "/admin/pilots")
        .await
        .assert_status(403);
}

#[tokio::test]
async fn groups_corporations_with_their_standing_and_member_characters() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    app.corporation(5000, Some("Blue Corp"), Some("BLUE"), None)
        .await;
    app.source_contact(5000, "corporation", 5.0).await;
    let pilot = app.user().await;
    let main = character(&app, "Main Pilot", Owner::User(pilot), Some(5000)).await;
    character(&app, "Alt Pilot", Owner::User(pilot), Some(5000)).await;
    // An alt outside the corporation is not part of its member list.
    character(&app, "Outside Alt", Owner::User(pilot), None).await;
    app.set_main(pilot, main).await;

    let response = app
        .page(&browser, "/admin/pilots?view=corporations&search=BLUE")
        .await;
    response.assert_status(200);
    let props = response.json();
    assert_eq!(props["characters"], json!(null));
    let groups = props["groups"]["data"].as_array().expect("groups");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["name"], "Blue Corp");
    assert_json_eq(
        &json!({ "value": 5, "source": "corporation" }),
        &groups[0]["standing"],
    );
    let members = groups[0]["characters"].as_array().expect("members");
    assert_eq!(members.len(), 2);
    assert_eq!(members[0]["name"], "Alt Pilot");
    assert_eq!(members[0]["account"]["name"], "Main Pilot");
    assert_eq!(members[0]["standing"]["value"], 5);
    assert_eq!(members[1]["is_main"], true);
    assert_eq!(props["filters"]["view"], "corporations");
}

#[tokio::test]
async fn groups_alliances_with_their_member_characters() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    app.alliance(6000, Some("Blue Alliance"), Some("BLUA"))
        .await;
    let pilot = app.user().await;
    app.character(NewCharacter {
        name: Some("Alliance Pilot"),
        owner: Owner::User(pilot),
        alliance_id: Some(6000),
        ..NewCharacter::default()
    })
    .await;

    let props = app
        .page(&browser, "/admin/pilots?view=alliances")
        .await
        .json();
    let groups = props["groups"]["data"].as_array().expect("groups");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["name"], "Blue Alliance");
    assert_eq!(groups[0]["standing"], json!(null));
    assert_eq!(groups[0]["characters"][0]["name"], "Alliance Pilot");
}

#[tokio::test]
async fn filters_group_views_by_character_name_on_the_server() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    app.corporation(5000, Some("Blue Corp"), None, None).await;
    let first = app.user().await;
    character(&app, "Jita Trader", Owner::User(first), Some(5000)).await;
    app.corporation(5001, Some("Other Corp"), None, None).await;
    let second = app.user().await;
    character(&app, "Amarr Hauler", Owner::User(second), Some(5001)).await;

    let props = app
        .page(&browser, "/admin/pilots?view=corporations&search=Jita")
        .await
        .json();
    let groups = props["groups"]["data"].as_array().expect("groups");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["name"], "Blue Corp");
}

#[tokio::test]
async fn marks_characters_and_corporations_covered_by_the_source_as_green_source_standings() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    app.corporation(5000, Some("Source Corp"), None, None).await;
    app.source("corporation", 5000).await;
    let pilot = app.user().await;
    character(&app, "Member Pilot", Owner::User(pilot), Some(5000)).await;

    let props = app.page(&browser, "/admin/pilots").await.json();
    let row = props["characters"]
        .as_array()
        .expect("characters")
        .iter()
        .find(|row| row["name"] == "Member Pilot")
        .expect("the member")
        .clone();
    assert_json_eq(
        &json!({ "value": 10, "source": "source" }),
        &row["standing"],
    );

    let props = app
        .page(&browser, "/admin/pilots?view=corporations")
        .await
        .json();
    assert_eq!(props["groups"]["data"][0]["name"], "Source Corp");
    assert_eq!(props["groups"]["data"][0]["standing"]["source"], "source");
}

#[tokio::test]
async fn excludes_corporations_without_registered_characters() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    app.corporation(5000, Some("Empty Corp"), None, None).await;
    character(&app, "Nobody's Pilot", Owner::Nobody, Some(5000)).await;

    let props = app
        .page(&browser, "/admin/pilots?view=corporations")
        .await
        .json();
    assert_eq!(props["groups"]["data"].as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn the_search_matches_percent_and_underscore_literally() {
    let mut app = TestApp::new().await;
    let browser = admin(&mut app).await;
    app.corporation(5000, Some("Blue Corp"), Some("BLUE"), None)
        .await;
    app.corporation(5001, Some("100% Corp"), Some("PCT"), None)
        .await;
    app.corporation(5002, Some("Under_Score Inc"), Some("US"), None)
        .await;
    let pilot = app.user().await;
    character(&app, "Blue Pilot", Owner::User(pilot), Some(5000)).await;
    character(&app, "Percent Pilot", Owner::User(pilot), Some(5001)).await;
    character(&app, "Under Pilot", Owner::User(pilot), Some(5002)).await;

    let names = |props: serde_json::Value| -> Vec<String> {
        props["groups"]["data"]
            .as_array()
            .expect("groups")
            .iter()
            .map(|group| group["name"].as_str().expect("a name").to_owned())
            .collect()
    };
    let percent = app
        .page(&browser, "/admin/pilots?view=corporations&search=%25")
        .await
        .json();
    assert_eq!(names(percent), vec!["100% Corp"]);
    let underscore = app
        .page(&browser, "/admin/pilots?view=corporations&search=_")
        .await
        .json();
    assert_eq!(names(underscore), vec!["Under_Score Inc"]);
}

//! The props of every page in the world (common::world): the shared props
//! once per kind of visitor, and each page's own props in full.

use serde_json::{Value, json};

use crate::common::world::{ADMIN, ADMIN_ALT, FRESH, MEMBER, NICOLAS, RED, TINA};
use crate::common::{APP_URL, Browser, TestApp, assert_json_eq};

/// A page's own props, without the shared ones every page carries.
fn own(props: &Value) -> Value {
    let mut own = props.clone();
    if let Some(map) = own.as_object_mut() {
        for key in ["name", "auth", "sidebarOpen", "flash"] {
            map.shift_remove(key);
        }
    }
    own
}

async fn page(app: &TestApp, browser: &Browser, path: &str) -> Value {
    let response = app.page(browser, path).await;
    response.assert_status(200);
    response.json()
}

/// The props every page carries, for a guest.
#[tokio::test]
async fn a_guest_gets_the_shared_props() {
    let app = TestApp::world().await;

    let props = app.page(&Browser::guest(), "/").await.json();

    assert_json_eq(
        &json!({
            "name": "Bluebook",
            "auth": {
                "user": null
            },
            "sidebarOpen": true,
            "flash": {
                "success": null,
                "error": null,
                "info": null
            }
        }),
        &props,
    );
}

/// The props every page carries, for a pilot with three characters.
#[tokio::test]
async fn a_pilot_gets_the_shared_props() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    let props = page(&app, &browser, "/").await;

    assert_json_eq(
        &json!({
            "name": "Bluebook",
            "auth": {
                "user": {
                    "id": 2,
                    "name": "Nicolas Pilot"
                },
                "is_admin": false,
                "source_unreadable": false,
                "reauth_characters": [
                    {
                        "id": 95000002,
                        "name": "Tina Pilot"
                    }
                ],
                "sync_scopes_url": "https://bluebook.nicolaskion.dev/eve?add_to_account=1&scopes=esi-characters.read_contacts.v1%2Cesi-characters.write_contacts.v1",
                "admin_scopes_url": null,
                "active_character": {
                    "id": 95000001,
                    "name": "Nicolas Pilot",
                    "corporation_id": 98000002,
                    "alliance_id": 99000001,
                    "corporation": {
                        "id": 98000002,
                        "name": "Blue Corp",
                        "ticker": "BLUC"
                    },
                    "alliance": {
                        "id": 99000001,
                        "name": "Blue Alliance",
                        "ticker": "BLUA"
                    }
                },
                "characters": [
                    {
                        "id": 95000001,
                        "name": "Nicolas Pilot",
                        "corporation_id": 98000002,
                        "alliance_id": 99000001,
                        "corporation": {
                            "id": 98000002,
                            "name": "Blue Corp",
                            "ticker": "BLUC"
                        },
                        "alliance": {
                            "id": 99000001,
                            "name": "Blue Alliance",
                            "ticker": "BLUA"
                        }
                    },
                    {
                        "id": 95000002,
                        "name": "Tina Pilot",
                        "corporation_id": 98000003,
                        "alliance_id": null,
                        "corporation": {
                            "id": 98000003,
                            "name": "Kion Trading Inc.",
                            "ticker": "KION"
                        },
                        "alliance": null
                    },
                    {
                        "id": 95000003,
                        "name": "Émile Pilot",
                        "corporation_id": 98000005,
                        "alliance_id": null,
                        "corporation": {
                            "id": 98000005,
                            "name": "Écho Industries",
                            "ticker": "ECHO"
                        },
                        "alliance": null
                    }
                ]
            },
            "sidebarOpen": true,
            "flash": {
                "success": null,
                "error": null,
                "info": null
            }
        }),
        &props,
    );
    assert_json_eq(&props, &page(&app, &browser, "/settings/appearance").await);
}

/// The admin is told so, and gets the admin grant link.
#[tokio::test]
async fn the_admin_gets_the_shared_props() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(1, ADMIN).await;

    let props = page(&app, &browser, "/").await;

    assert_json_eq(
        &json!({
            "name": "Bluebook",
            "auth": {
                "user": {
                    "id": 1,
                    "name": "Admin Pilot"
                },
                "is_admin": true,
                "source_unreadable": false,
                "reauth_characters": [],
                "sync_scopes_url": "https://bluebook.nicolaskion.dev/eve?add_to_account=1&scopes=esi-characters.read_contacts.v1%2Cesi-characters.write_contacts.v1",
                "admin_scopes_url": "https://bluebook.nicolaskion.dev/eve?add_to_account=1&scopes=publicData%2Cesi-characters.read_contacts.v1%2Cesi-characters.write_contacts.v1%2Cesi-corporations.read_contacts.v1%2Cesi-alliances.read_contacts.v1%2Cesi-mail.send_mail.v1",
                "active_character": {
                    "id": 2112000001,
                    "name": "Admin Pilot",
                    "corporation_id": 98000001,
                    "alliance_id": 99000002,
                    "corporation": {
                        "id": 98000001,
                        "name": "Home Corp",
                        "ticker": "HOMC"
                    },
                    "alliance": {
                        "id": 99000002,
                        "name": "Home Alliance",
                        "ticker": "HOME"
                    }
                },
                "characters": [
                    {
                        "id": 2112000001,
                        "name": "Admin Pilot",
                        "corporation_id": 98000001,
                        "alliance_id": 99000002,
                        "corporation": {
                            "id": 98000001,
                            "name": "Home Corp",
                            "ticker": "HOMC"
                        },
                        "alliance": {
                            "id": 99000002,
                            "name": "Home Alliance",
                            "ticker": "HOME"
                        }
                    },
                    {
                        "id": 2112000002,
                        "name": "admin alt",
                        "corporation_id": 98000003,
                        "alliance_id": null,
                        "corporation": {
                            "id": 98000003,
                            "name": "Kion Trading Inc.",
                            "ticker": "KION"
                        },
                        "alliance": null
                    }
                ]
            },
            "sidebarOpen": true,
            "flash": {
                "success": null,
                "error": null,
                "info": null
            }
        }),
        &props,
    );
}

/// The source, its standings with what makes them redundant, the characters and what they can request.
#[tokio::test]
async fn the_dashboard_of_a_pilot() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(2, NICOLAS).await;

    let props = page(&app, &browser, "/dashboard").await;

    assert_json_eq(&page(&app, &browser, "/").await["auth"], &props["auth"]);
    assert_json_eq(
        &json!({
            "showSetupWizard": false,
            "source": {
                "type": "alliance",
                "entity_id": 99000002,
                "entity_name": "Home Alliance",
                "last_synced_at": "2026-06-25T13:58:00+00:00"
            },
            "canViewStandings": true,
            "standings": [
                {
                    "contact_id": 99000001,
                    "contact_type": "alliance",
                    "name": "Blue Alliance",
                    "standing": 10,
                    "redundant_via": null
                },
                {
                    "contact_id": 98000002,
                    "contact_type": "corporation",
                    "name": "Blue Corp",
                    "standing": 10,
                    "redundant_via": {
                        "contact_type": "alliance",
                        "contact_id": 99000001,
                        "name": "Blue Alliance"
                    }
                },
                {
                    "contact_id": 95000100,
                    "contact_type": "character",
                    "name": "Contact One",
                    "standing": 5,
                    "redundant_via": {
                        "contact_type": "corporation",
                        "contact_id": 98000002,
                        "name": "Blue Corp"
                    }
                },
                {
                    "contact_id": 98000099,
                    "contact_type": "corporation",
                    "name": null,
                    "standing": 5,
                    "redundant_via": null
                },
                {
                    "contact_id": 95000002,
                    "contact_type": "character",
                    "name": "Tina Pilot",
                    "standing": 0,
                    "redundant_via": null
                },
                {
                    "contact_id": 98000003,
                    "contact_type": "corporation",
                    "name": "Kion Trading Inc.",
                    "standing": -5,
                    "redundant_via": null
                },
                {
                    "contact_id": 500001,
                    "contact_type": "faction",
                    "name": "Caldari State",
                    "standing": -5,
                    "redundant_via": null
                },
                {
                    "contact_id": 99000003,
                    "contact_type": "alliance",
                    "name": "Red Alliance",
                    "standing": -10,
                    "redundant_via": null
                },
                {
                    "contact_id": 95000101,
                    "contact_type": "character",
                    "name": "Spy Pilot",
                    "standing": -10,
                    "redundant_via": {
                        "contact_type": "corporation",
                        "contact_id": 98000004,
                        "name": "Red Corp"
                    }
                },
                {
                    "contact_id": 98000004,
                    "contact_type": "corporation",
                    "name": "Red Corp",
                    "standing": -10,
                    "redundant_via": {
                        "contact_type": "alliance",
                        "contact_id": 99000003,
                        "name": "Red Alliance"
                    }
                }
            ],
            "characters": [
                {
                    "id": 95000001,
                    "name": "Nicolas Pilot",
                    "is_main": true,
                    "should_sync": true,
                    "has_write_scope": true,
                    "inherits_source": false,
                    "synced_contacts_count": 3,
                    "corporation": {
                        "id": 98000002,
                        "name": "Blue Corp",
                        "ticker": "BLUC"
                    },
                    "alliance": {
                        "id": 99000001,
                        "name": "Blue Alliance",
                        "ticker": "BLUA"
                    }
                },
                {
                    "id": 95000002,
                    "name": "Tina Pilot",
                    "is_main": false,
                    "should_sync": true,
                    "has_write_scope": false,
                    "inherits_source": false,
                    "synced_contacts_count": 0,
                    "corporation": {
                        "id": 98000003,
                        "name": "Kion Trading Inc.",
                        "ticker": "KION"
                    },
                    "alliance": null
                },
                {
                    "id": 95000003,
                    "name": "Émile Pilot",
                    "is_main": false,
                    "should_sync": false,
                    "has_write_scope": false,
                    "inherits_source": false,
                    "synced_contacts_count": 0,
                    "corporation": {
                        "id": 98000005,
                        "name": "Écho Industries",
                        "ticker": "ECHO"
                    },
                    "alliance": null
                }
            ],
            "requestableOptions": [
                {
                    "type": "character",
                    "id": 95000001,
                    "name": "Nicolas Pilot",
                    "via_character_id": 95000001,
                    "status": "done"
                },
                {
                    "type": "corporation",
                    "id": 98000002,
                    "name": "Blue Corp",
                    "via_character_id": 95000001,
                    "status": null
                },
                {
                    "type": "alliance",
                    "id": 99000001,
                    "name": "Blue Alliance",
                    "via_character_id": 95000001,
                    "status": null
                },
                {
                    "type": "character",
                    "id": 95000002,
                    "name": "Tina Pilot",
                    "via_character_id": 95000002,
                    "status": null
                },
                {
                    "type": "corporation",
                    "id": 98000003,
                    "name": "Kion Trading Inc.",
                    "via_character_id": 95000002,
                    "status": "pending"
                },
                {
                    "type": "character",
                    "id": 95000003,
                    "name": "Émile Pilot",
                    "via_character_id": 95000003,
                    "status": null
                },
                {
                    "type": "corporation",
                    "id": 98000005,
                    "name": "Écho Industries",
                    "via_character_id": 95000003,
                    "status": "pending"
                }
            ]
        }),
        &own(&props),
    );
}

/// A session without an active character acts as the main.
#[tokio::test]
async fn the_dashboard_without_an_active_character_shows_the_main() {
    let app = TestApp::world().await;

    let without = page(&app, &app.sign_in(2).await, "/dashboard").await;

    let with_main = page(&app, &app.sign_in_as(2, NICOLAS).await, "/dashboard").await;
    assert_json_eq(&with_main, &without);
}

/// Only the active character changes.
#[tokio::test]
async fn the_dashboard_with_an_alt_active() {
    let app = TestApp::world().await;

    let props = page(&app, &app.sign_in_as(2, TINA).await, "/dashboard").await;

    let main = page(&app, &app.sign_in_as(2, NICOLAS).await, "/dashboard").await;
    assert_json_eq(&own(&main), &own(&props));
    assert_json_eq(
        &json!({
            "id": 95000002,
            "name": "Tina Pilot",
            "corporation_id": 98000003,
            "alliance_id": null,
            "corporation": {
                "id": 98000003,
                "name": "Kion Trading Inc.",
                "ticker": "KION"
            },
            "alliance": null
        }),
        &props["auth"]["active_character"],
    );
}

/// A pilot who has not picked a main sees the wizard; Fresh Pilot is not covered by the source.
#[tokio::test]
async fn the_dashboard_of_a_pilot_who_has_not_picked_a_main() {
    let app = TestApp::world().await;

    let props = page(&app, &app.sign_in_as(3, FRESH).await, "/dashboard").await;

    assert_json_eq(
        &json!({
            "showSetupWizard": true,
            "source": {
                "type": "alliance",
                "entity_id": 99000002,
                "entity_name": "Home Alliance",
                "last_synced_at": "2026-06-25T13:58:00+00:00"
            },
            "canViewStandings": false,
            "standings": null,
            "characters": [
                {
                    "id": 95000010,
                    "name": "Fresh Pilot",
                    "is_main": false,
                    "should_sync": false,
                    "has_write_scope": false,
                    "inherits_source": false,
                    "synced_contacts_count": 0,
                    "corporation": {
                        "id": 98000003,
                        "name": "Kion Trading Inc.",
                        "ticker": "KION"
                    },
                    "alliance": null
                }
            ],
            "requestableOptions": [
                {
                    "type": "character",
                    "id": 95000010,
                    "name": "Fresh Pilot",
                    "via_character_id": 95000010,
                    "status": "pending"
                },
                {
                    "type": "corporation",
                    "id": 98000003,
                    "name": "Kion Trading Inc.",
                    "via_character_id": 95000010,
                    "status": "pending"
                }
            ]
        }),
        &own(&props),
    );
}

/// A red pilot does not see the standings but can still request.
#[tokio::test]
async fn the_dashboard_of_a_red_pilot() {
    let app = TestApp::world().await;

    let props = page(&app, &app.sign_in_as(4, RED).await, "/dashboard").await;

    assert_json_eq(
        &json!({
            "showSetupWizard": false,
            "source": {
                "type": "alliance",
                "entity_id": 99000002,
                "entity_name": "Home Alliance",
                "last_synced_at": "2026-06-25T13:58:00+00:00"
            },
            "canViewStandings": false,
            "standings": null,
            "characters": [
                {
                    "id": 95000020,
                    "name": "Red Pilot",
                    "is_main": false,
                    "should_sync": true,
                    "has_write_scope": true,
                    "inherits_source": false,
                    "synced_contacts_count": 0,
                    "corporation": {
                        "id": 98000004,
                        "name": "Red Corp",
                        "ticker": "REDC"
                    },
                    "alliance": {
                        "id": 99000003,
                        "name": "Red Alliance",
                        "ticker": "RED"
                    }
                }
            ],
            "requestableOptions": [
                {
                    "type": "character",
                    "id": 95000020,
                    "name": "Red Pilot",
                    "via_character_id": 95000020,
                    "status": null
                },
                {
                    "type": "corporation",
                    "id": 98000004,
                    "name": "Red Corp",
                    "via_character_id": 95000020,
                    "status": null
                },
                {
                    "type": "alliance",
                    "id": 99000003,
                    "name": "Red Alliance",
                    "via_character_id": 95000020,
                    "status": "rejected"
                }
            ]
        }),
        &own(&props),
    );
}

/// A member of the source alliance inherits the source in game.
#[tokio::test]
async fn the_dashboard_of_a_member_of_the_source() {
    let app = TestApp::world().await;

    let props = page(&app, &app.sign_in_as(5, MEMBER).await, "/dashboard").await;

    assert_json_eq(
        &json!({
            "showSetupWizard": false,
            "source": {
                "type": "alliance",
                "entity_id": 99000002,
                "entity_name": "Home Alliance",
                "last_synced_at": "2026-06-25T13:58:00+00:00"
            },
            "canViewStandings": true,
            "standings": [
                {
                    "contact_id": 99000001,
                    "contact_type": "alliance",
                    "name": "Blue Alliance",
                    "standing": 10,
                    "redundant_via": null
                },
                {
                    "contact_id": 98000002,
                    "contact_type": "corporation",
                    "name": "Blue Corp",
                    "standing": 10,
                    "redundant_via": {
                        "contact_type": "alliance",
                        "contact_id": 99000001,
                        "name": "Blue Alliance"
                    }
                },
                {
                    "contact_id": 95000100,
                    "contact_type": "character",
                    "name": "Contact One",
                    "standing": 5,
                    "redundant_via": {
                        "contact_type": "corporation",
                        "contact_id": 98000002,
                        "name": "Blue Corp"
                    }
                },
                {
                    "contact_id": 98000099,
                    "contact_type": "corporation",
                    "name": null,
                    "standing": 5,
                    "redundant_via": null
                },
                {
                    "contact_id": 95000002,
                    "contact_type": "character",
                    "name": "Tina Pilot",
                    "standing": 0,
                    "redundant_via": null
                },
                {
                    "contact_id": 98000003,
                    "contact_type": "corporation",
                    "name": "Kion Trading Inc.",
                    "standing": -5,
                    "redundant_via": null
                },
                {
                    "contact_id": 500001,
                    "contact_type": "faction",
                    "name": "Caldari State",
                    "standing": -5,
                    "redundant_via": null
                },
                {
                    "contact_id": 99000003,
                    "contact_type": "alliance",
                    "name": "Red Alliance",
                    "standing": -10,
                    "redundant_via": null
                },
                {
                    "contact_id": 95000101,
                    "contact_type": "character",
                    "name": "Spy Pilot",
                    "standing": -10,
                    "redundant_via": {
                        "contact_type": "corporation",
                        "contact_id": 98000004,
                        "name": "Red Corp"
                    }
                },
                {
                    "contact_id": 98000004,
                    "contact_type": "corporation",
                    "name": "Red Corp",
                    "standing": -10,
                    "redundant_via": {
                        "contact_type": "alliance",
                        "contact_id": 99000003,
                        "name": "Red Alliance"
                    }
                }
            ],
            "characters": [
                {
                    "id": 95000030,
                    "name": "Member Pilot",
                    "is_main": true,
                    "should_sync": true,
                    "has_write_scope": true,
                    "inherits_source": true,
                    "synced_contacts_count": 0,
                    "corporation": {
                        "id": 98000001,
                        "name": "Home Corp",
                        "ticker": "HOMC"
                    },
                    "alliance": {
                        "id": 99000002,
                        "name": "Home Alliance",
                        "ticker": "HOME"
                    }
                }
            ],
            "requestableOptions": []
        }),
        &own(&props),
    );
}

/// Without a source there is nothing to show or request.
#[tokio::test]
async fn the_dashboard_without_a_source() {
    let app = TestApp::world().await;
    app.exec("delete from standings_sources").await;

    let props = page(&app, &app.sign_in_as(2, NICOLAS).await, "/dashboard").await;

    assert_json_eq(
        &json!({
            "showSetupWizard": false,
            "source": null,
            "canViewStandings": false,
            "standings": null,
            "characters": [
                {
                    "id": 95000001,
                    "name": "Nicolas Pilot",
                    "is_main": true,
                    "should_sync": true,
                    "has_write_scope": true,
                    "inherits_source": false,
                    "synced_contacts_count": 3,
                    "corporation": {
                        "id": 98000002,
                        "name": "Blue Corp",
                        "ticker": "BLUC"
                    },
                    "alliance": {
                        "id": 99000001,
                        "name": "Blue Alliance",
                        "ticker": "BLUA"
                    }
                },
                {
                    "id": 95000002,
                    "name": "Tina Pilot",
                    "is_main": false,
                    "should_sync": true,
                    "has_write_scope": false,
                    "inherits_source": false,
                    "synced_contacts_count": 0,
                    "corporation": {
                        "id": 98000003,
                        "name": "Kion Trading Inc.",
                        "ticker": "KION"
                    },
                    "alliance": null
                },
                {
                    "id": 95000003,
                    "name": "Émile Pilot",
                    "is_main": false,
                    "should_sync": false,
                    "has_write_scope": false,
                    "inherits_source": false,
                    "synced_contacts_count": 0,
                    "corporation": {
                        "id": 98000005,
                        "name": "Écho Industries",
                        "ticker": "ECHO"
                    },
                    "alliance": null
                }
            ],
            "requestableOptions": []
        }),
        &own(&props),
    );
}

/// The source names the character.
#[tokio::test]
async fn the_dashboard_with_a_character_as_the_source() {
    let app = TestApp::world().await;
    app.exec("update standings_sources set type = 'character', entity_id = 2112000001")
        .await;

    let props = page(&app, &app.sign_in_as(2, NICOLAS).await, "/dashboard").await;

    assert_json_eq(
        &json!({
            "type": "character",
            "entity_id": 2112000001,
            "entity_name": "Admin Pilot",
            "last_synced_at": "2026-06-25T13:58:00+00:00"
        }),
        &props["source"],
    );
}

/// The admin sees the standings; the admin character does not sync.
#[tokio::test]
async fn the_dashboard_of_the_admin() {
    let app = TestApp::world().await;

    let props = page(&app, &app.sign_in_as(1, ADMIN).await, "/dashboard").await;

    assert_json_eq(
        &json!({
            "showSetupWizard": false,
            "source": {
                "type": "alliance",
                "entity_id": 99000002,
                "entity_name": "Home Alliance",
                "last_synced_at": "2026-06-25T13:58:00+00:00"
            },
            "canViewStandings": true,
            "standings": [
                {
                    "contact_id": 99000001,
                    "contact_type": "alliance",
                    "name": "Blue Alliance",
                    "standing": 10,
                    "redundant_via": null
                },
                {
                    "contact_id": 98000002,
                    "contact_type": "corporation",
                    "name": "Blue Corp",
                    "standing": 10,
                    "redundant_via": {
                        "contact_type": "alliance",
                        "contact_id": 99000001,
                        "name": "Blue Alliance"
                    }
                },
                {
                    "contact_id": 95000100,
                    "contact_type": "character",
                    "name": "Contact One",
                    "standing": 5,
                    "redundant_via": {
                        "contact_type": "corporation",
                        "contact_id": 98000002,
                        "name": "Blue Corp"
                    }
                },
                {
                    "contact_id": 98000099,
                    "contact_type": "corporation",
                    "name": null,
                    "standing": 5,
                    "redundant_via": null
                },
                {
                    "contact_id": 95000002,
                    "contact_type": "character",
                    "name": "Tina Pilot",
                    "standing": 0,
                    "redundant_via": null
                },
                {
                    "contact_id": 98000003,
                    "contact_type": "corporation",
                    "name": "Kion Trading Inc.",
                    "standing": -5,
                    "redundant_via": null
                },
                {
                    "contact_id": 500001,
                    "contact_type": "faction",
                    "name": "Caldari State",
                    "standing": -5,
                    "redundant_via": null
                },
                {
                    "contact_id": 99000003,
                    "contact_type": "alliance",
                    "name": "Red Alliance",
                    "standing": -10,
                    "redundant_via": null
                },
                {
                    "contact_id": 95000101,
                    "contact_type": "character",
                    "name": "Spy Pilot",
                    "standing": -10,
                    "redundant_via": {
                        "contact_type": "corporation",
                        "contact_id": 98000004,
                        "name": "Red Corp"
                    }
                },
                {
                    "contact_id": 98000004,
                    "contact_type": "corporation",
                    "name": "Red Corp",
                    "standing": -10,
                    "redundant_via": {
                        "contact_type": "alliance",
                        "contact_id": 99000003,
                        "name": "Red Alliance"
                    }
                }
            ],
            "characters": [
                {
                    "id": 2112000001,
                    "name": "Admin Pilot",
                    "is_main": true,
                    "should_sync": false,
                    "has_write_scope": true,
                    "inherits_source": true,
                    "synced_contacts_count": 0,
                    "corporation": {
                        "id": 98000001,
                        "name": "Home Corp",
                        "ticker": "HOMC"
                    },
                    "alliance": {
                        "id": 99000002,
                        "name": "Home Alliance",
                        "ticker": "HOME"
                    }
                },
                {
                    "id": 2112000002,
                    "name": "admin alt",
                    "is_main": false,
                    "should_sync": true,
                    "has_write_scope": true,
                    "inherits_source": false,
                    "synced_contacts_count": 0,
                    "corporation": {
                        "id": 98000003,
                        "name": "Kion Trading Inc.",
                        "ticker": "KION"
                    },
                    "alliance": null
                }
            ],
            "requestableOptions": [
                {
                    "type": "character",
                    "id": 2112000002,
                    "name": "admin alt",
                    "via_character_id": 2112000002,
                    "status": null
                },
                {
                    "type": "corporation",
                    "id": 98000003,
                    "name": "Kion Trading Inc.",
                    "via_character_id": 2112000002,
                    "status": "pending"
                }
            ]
        }),
        &own(&props),
    );
}

/// The source, the counts and the latest requests.
#[tokio::test]
async fn the_admin_overview() {
    let app = TestApp::world().await;

    let props = page(&app, &app.sign_in_as(1, ADMIN).await, "/admin").await;

    assert_json_eq(
        &json!({
            "source": {
                "type": "alliance",
                "entity_id": 99000002,
                "entity_name": "Home Alliance",
                "last_synced_at": "2026-06-25T13:58:00+00:00"
            },
            "stats": {
                "pending_requests": 3,
                "pilots": 5,
                "syncing_characters": 5,
                "source_contacts": 10
            },
            "recentRequests": [
                {
                    "id": 2,
                    "status": "pending",
                    "created_at": "2026-06-25T13:30:00+00:00",
                    "subject": {
                        "type": "corporation",
                        "id": 98000003,
                        "name": "Kion Trading Inc."
                    },
                    "requested_by": "Tina Pilot",
                    "effective_standing": {
                        "standing": -5,
                        "source": "direct",
                        "via_type": "corporation",
                        "via_id": 98000003,
                        "via_name": "Kion Trading Inc."
                    }
                },
                {
                    "id": 5,
                    "status": "pending",
                    "created_at": "2026-06-25T13:15:00+00:00",
                    "subject": {
                        "type": "corporation",
                        "id": 98000005,
                        "name": "Écho Industries"
                    },
                    "requested_by": "Émile Pilot",
                    "effective_standing": null
                },
                {
                    "id": 1,
                    "status": "pending",
                    "created_at": "2026-06-25T13:00:00+00:00",
                    "subject": {
                        "type": "character",
                        "id": 95000010,
                        "name": "Fresh Pilot"
                    },
                    "requested_by": "Fresh Pilot",
                    "effective_standing": {
                        "standing": -5,
                        "source": "corporation",
                        "via_type": "corporation",
                        "via_id": 98000003,
                        "via_name": "Kion Trading Inc."
                    }
                }
            ]
        }),
        &own(&props),
    );
}

#[tokio::test]
async fn the_admin_overview_without_a_source() {
    let app = TestApp::world().await;
    app.exec("delete from standings_sources").await;

    let props = page(&app, &app.sign_in_as(1, ADMIN).await, "/admin").await;

    assert_json_eq(&json!(null), &props["source"]);
}

#[tokio::test]
async fn the_admin_is_warned_when_no_admin_token_can_read_the_source() {
    let app = TestApp::world().await;
    app.exec("delete from esi_tokens where character_id = 2112000001")
        .await;

    let props = page(&app, &app.sign_in_as(1, ADMIN).await, "/admin").await;

    assert_eq!(props["auth"]["source_unreadable"], true);
}

/// Every registered character with its account and effective standing.
#[tokio::test]
async fn the_pilots_page() {
    let app = TestApp::world().await;

    let props = page(&app, &app.sign_in_as(1, ADMIN).await, "/admin/pilots").await;

    assert_json_eq(
        &json!({
            "characters": [
                {
                    "id": 2112000002,
                    "name": "admin alt",
                    "is_main": false,
                    "account": {
                        "id": 1,
                        "name": "Admin Pilot"
                    },
                    "standing": {
                        "value": -5,
                        "source": "corporation"
                    },
                    "corporation": {
                        "id": 98000003,
                        "name": "Kion Trading Inc.",
                        "ticker": "KION"
                    },
                    "alliance": null
                },
                {
                    "id": 2112000001,
                    "name": "Admin Pilot",
                    "is_main": true,
                    "account": {
                        "id": 1,
                        "name": "Admin Pilot"
                    },
                    "standing": {
                        "value": 10,
                        "source": "source"
                    },
                    "corporation": {
                        "id": 98000001,
                        "name": "Home Corp",
                        "ticker": "HOMC"
                    },
                    "alliance": {
                        "id": 99000002,
                        "name": "Home Alliance",
                        "ticker": "HOME"
                    }
                },
                {
                    "id": 95000003,
                    "name": "Émile Pilot",
                    "is_main": false,
                    "account": {
                        "id": 2,
                        "name": "Nicolas Pilot"
                    },
                    "standing": null,
                    "corporation": {
                        "id": 98000005,
                        "name": "Écho Industries",
                        "ticker": "ECHO"
                    },
                    "alliance": null
                },
                {
                    "id": 95000010,
                    "name": "Fresh Pilot",
                    "is_main": false,
                    "account": {
                        "id": 3,
                        "name": "Fresh Pilot"
                    },
                    "standing": {
                        "value": -5,
                        "source": "corporation"
                    },
                    "corporation": {
                        "id": 98000003,
                        "name": "Kion Trading Inc.",
                        "ticker": "KION"
                    },
                    "alliance": null
                },
                {
                    "id": 95000030,
                    "name": "Member Pilot",
                    "is_main": true,
                    "account": {
                        "id": 5,
                        "name": "Member Pilot"
                    },
                    "standing": {
                        "value": 10,
                        "source": "source"
                    },
                    "corporation": {
                        "id": 98000001,
                        "name": "Home Corp",
                        "ticker": "HOMC"
                    },
                    "alliance": {
                        "id": 99000002,
                        "name": "Home Alliance",
                        "ticker": "HOME"
                    }
                },
                {
                    "id": 95000001,
                    "name": "Nicolas Pilot",
                    "is_main": true,
                    "account": {
                        "id": 2,
                        "name": "Nicolas Pilot"
                    },
                    "standing": {
                        "value": 10,
                        "source": "corporation"
                    },
                    "corporation": {
                        "id": 98000002,
                        "name": "Blue Corp",
                        "ticker": "BLUC"
                    },
                    "alliance": {
                        "id": 99000001,
                        "name": "Blue Alliance",
                        "ticker": "BLUA"
                    }
                },
                {
                    "id": 95000020,
                    "name": "Red Pilot",
                    "is_main": false,
                    "account": {
                        "id": 4,
                        "name": "Red Pilot"
                    },
                    "standing": {
                        "value": -10,
                        "source": "corporation"
                    },
                    "corporation": {
                        "id": 98000004,
                        "name": "Red Corp",
                        "ticker": "REDC"
                    },
                    "alliance": {
                        "id": 99000003,
                        "name": "Red Alliance",
                        "ticker": "RED"
                    }
                },
                {
                    "id": 95000002,
                    "name": "Tina Pilot",
                    "is_main": false,
                    "account": {
                        "id": 2,
                        "name": "Nicolas Pilot"
                    },
                    "standing": {
                        "value": 0,
                        "source": "character"
                    },
                    "corporation": {
                        "id": 98000003,
                        "name": "Kion Trading Inc.",
                        "ticker": "KION"
                    },
                    "alliance": null
                }
            ],
            "groups": null,
            "filters": {
                "search": "",
                "view": "characters"
            }
        }),
        &own(&props),
    );
}

/// The alliances with registered characters, with their members.
#[tokio::test]
async fn the_pilots_page_by_alliance() {
    let app = TestApp::world().await;

    let props = page(
        &app,
        &app.sign_in_as(1, ADMIN).await,
        "/admin/pilots?view=alliances",
    )
    .await;

    assert_json_eq(
        &json!({
            "characters": null,
            "groups": {
                "current_page": 1,
                "data": [
                    {
                        "id": 99000001,
                        "name": "Blue Alliance",
                        "ticker": "BLUA",
                        "standing": {
                            "value": 10,
                            "source": "alliance"
                        },
                        "characters": [
                            {
                                "id": 95000001,
                                "name": "Nicolas Pilot",
                                "is_main": true,
                                "account": {
                                    "id": 2,
                                    "name": "Nicolas Pilot"
                                },
                                "standing": {
                                    "value": 10,
                                    "source": "corporation"
                                },
                                "corporation": {
                                    "id": 98000002,
                                    "name": "Blue Corp",
                                    "ticker": "BLUC"
                                },
                                "alliance": {
                                    "id": 99000001,
                                    "name": "Blue Alliance",
                                    "ticker": "BLUA"
                                }
                            }
                        ]
                    },
                    {
                        "id": 99000002,
                        "name": "Home Alliance",
                        "ticker": "HOME",
                        "standing": {
                            "value": 10,
                            "source": "source"
                        },
                        "characters": [
                            {
                                "id": 2112000001,
                                "name": "Admin Pilot",
                                "is_main": true,
                                "account": {
                                    "id": 1,
                                    "name": "Admin Pilot"
                                },
                                "standing": {
                                    "value": 10,
                                    "source": "source"
                                },
                                "corporation": {
                                    "id": 98000001,
                                    "name": "Home Corp",
                                    "ticker": "HOMC"
                                },
                                "alliance": {
                                    "id": 99000002,
                                    "name": "Home Alliance",
                                    "ticker": "HOME"
                                }
                            },
                            {
                                "id": 95000030,
                                "name": "Member Pilot",
                                "is_main": true,
                                "account": {
                                    "id": 5,
                                    "name": "Member Pilot"
                                },
                                "standing": {
                                    "value": 10,
                                    "source": "source"
                                },
                                "corporation": {
                                    "id": 98000001,
                                    "name": "Home Corp",
                                    "ticker": "HOMC"
                                },
                                "alliance": {
                                    "id": 99000002,
                                    "name": "Home Alliance",
                                    "ticker": "HOME"
                                }
                            }
                        ]
                    },
                    {
                        "id": 99000003,
                        "name": "Red Alliance",
                        "ticker": "RED",
                        "standing": {
                            "value": -10,
                            "source": "alliance"
                        },
                        "characters": [
                            {
                                "id": 95000020,
                                "name": "Red Pilot",
                                "is_main": false,
                                "account": {
                                    "id": 4,
                                    "name": "Red Pilot"
                                },
                                "standing": {
                                    "value": -10,
                                    "source": "corporation"
                                },
                                "corporation": {
                                    "id": 98000004,
                                    "name": "Red Corp",
                                    "ticker": "REDC"
                                },
                                "alliance": {
                                    "id": 99000003,
                                    "name": "Red Alliance",
                                    "ticker": "RED"
                                }
                            }
                        ]
                    }
                ],
                "first_page_url": "https://bluebook.nicolaskion.dev/admin/pilots?view=alliances&page=1",
                "from": 1,
                "last_page": 1,
                "last_page_url": "https://bluebook.nicolaskion.dev/admin/pilots?view=alliances&page=1",
                "links": [
                    {
                        "url": null,
                        "label": "&laquo; Previous",
                        "page": null,
                        "active": false
                    },
                    {
                        "url": "https://bluebook.nicolaskion.dev/admin/pilots?view=alliances&page=1",
                        "label": "1",
                        "page": 1,
                        "active": true
                    },
                    {
                        "url": null,
                        "label": "Next &raquo;",
                        "page": null,
                        "active": false
                    }
                ],
                "next_page_url": null,
                "path": "https://bluebook.nicolaskion.dev/admin/pilots",
                "per_page": 20,
                "prev_page_url": null,
                "to": 3,
                "total": 3
            },
            "filters": {
                "search": "",
                "view": "alliances"
            }
        }),
        &own(&props),
    );
}

/// The alliance view searches tickers too.
#[tokio::test]
async fn the_alliance_view_searches_tickers() {
    let app = TestApp::world().await;

    let props = page(
        &app,
        &app.sign_in_as(1, ADMIN).await,
        "/admin/pilots?view=alliances&search=blua",
    )
    .await;

    assert_json_eq(
        &json!({
            "characters": null,
            "groups": {
                "current_page": 1,
                "data": [
                    {
                        "id": 99000001,
                        "name": "Blue Alliance",
                        "ticker": "BLUA",
                        "standing": {
                            "value": 10,
                            "source": "alliance"
                        },
                        "characters": [
                            {
                                "id": 95000001,
                                "name": "Nicolas Pilot",
                                "is_main": true,
                                "account": {
                                    "id": 2,
                                    "name": "Nicolas Pilot"
                                },
                                "standing": {
                                    "value": 10,
                                    "source": "corporation"
                                },
                                "corporation": {
                                    "id": 98000002,
                                    "name": "Blue Corp",
                                    "ticker": "BLUC"
                                },
                                "alliance": {
                                    "id": 99000001,
                                    "name": "Blue Alliance",
                                    "ticker": "BLUA"
                                }
                            }
                        ]
                    }
                ],
                "first_page_url": "https://bluebook.nicolaskion.dev/admin/pilots?view=alliances&search=blua&page=1",
                "from": 1,
                "last_page": 1,
                "last_page_url": "https://bluebook.nicolaskion.dev/admin/pilots?view=alliances&search=blua&page=1",
                "links": [
                    {
                        "url": null,
                        "label": "&laquo; Previous",
                        "page": null,
                        "active": false
                    },
                    {
                        "url": "https://bluebook.nicolaskion.dev/admin/pilots?view=alliances&search=blua&page=1",
                        "label": "1",
                        "page": 1,
                        "active": true
                    },
                    {
                        "url": null,
                        "label": "Next &raquo;",
                        "page": null,
                        "active": false
                    }
                ],
                "next_page_url": null,
                "path": "https://bluebook.nicolaskion.dev/admin/pilots",
                "per_page": 20,
                "prev_page_url": null,
                "to": 1,
                "total": 1
            },
            "filters": {
                "search": "blua",
                "view": "alliances"
            }
        }),
        &own(&props),
    );
}

/// The corporations with registered characters, with their members.
#[tokio::test]
async fn the_pilots_page_by_corporation() {
    let app = TestApp::world().await;

    let props = page(
        &app,
        &app.sign_in_as(1, ADMIN).await,
        "/admin/pilots?view=corporations",
    )
    .await;

    assert_json_eq(
        &json!({
            "characters": null,
            "groups": {
                "current_page": 1,
                "data": [
                    {
                        "id": 98000002,
                        "name": "Blue Corp",
                        "ticker": "BLUC",
                        "standing": {
                            "value": 10,
                            "source": "corporation"
                        },
                        "characters": [
                            {
                                "id": 95000001,
                                "name": "Nicolas Pilot",
                                "is_main": true,
                                "account": {
                                    "id": 2,
                                    "name": "Nicolas Pilot"
                                },
                                "standing": {
                                    "value": 10,
                                    "source": "corporation"
                                },
                                "corporation": {
                                    "id": 98000002,
                                    "name": "Blue Corp",
                                    "ticker": "BLUC"
                                },
                                "alliance": {
                                    "id": 99000001,
                                    "name": "Blue Alliance",
                                    "ticker": "BLUA"
                                }
                            }
                        ]
                    },
                    {
                        "id": 98000005,
                        "name": "Écho Industries",
                        "ticker": "ECHO",
                        "standing": null,
                        "characters": [
                            {
                                "id": 95000003,
                                "name": "Émile Pilot",
                                "is_main": false,
                                "account": {
                                    "id": 2,
                                    "name": "Nicolas Pilot"
                                },
                                "standing": null,
                                "corporation": {
                                    "id": 98000005,
                                    "name": "Écho Industries",
                                    "ticker": "ECHO"
                                },
                                "alliance": null
                            }
                        ]
                    },
                    {
                        "id": 98000001,
                        "name": "Home Corp",
                        "ticker": "HOMC",
                        "standing": {
                            "value": 10,
                            "source": "source"
                        },
                        "characters": [
                            {
                                "id": 2112000001,
                                "name": "Admin Pilot",
                                "is_main": true,
                                "account": {
                                    "id": 1,
                                    "name": "Admin Pilot"
                                },
                                "standing": {
                                    "value": 10,
                                    "source": "source"
                                },
                                "corporation": {
                                    "id": 98000001,
                                    "name": "Home Corp",
                                    "ticker": "HOMC"
                                },
                                "alliance": {
                                    "id": 99000002,
                                    "name": "Home Alliance",
                                    "ticker": "HOME"
                                }
                            },
                            {
                                "id": 95000030,
                                "name": "Member Pilot",
                                "is_main": true,
                                "account": {
                                    "id": 5,
                                    "name": "Member Pilot"
                                },
                                "standing": {
                                    "value": 10,
                                    "source": "source"
                                },
                                "corporation": {
                                    "id": 98000001,
                                    "name": "Home Corp",
                                    "ticker": "HOMC"
                                },
                                "alliance": {
                                    "id": 99000002,
                                    "name": "Home Alliance",
                                    "ticker": "HOME"
                                }
                            }
                        ]
                    },
                    {
                        "id": 98000003,
                        "name": "Kion Trading Inc.",
                        "ticker": "KION",
                        "standing": {
                            "value": -5,
                            "source": "corporation"
                        },
                        "characters": [
                            {
                                "id": 2112000002,
                                "name": "admin alt",
                                "is_main": false,
                                "account": {
                                    "id": 1,
                                    "name": "Admin Pilot"
                                },
                                "standing": {
                                    "value": -5,
                                    "source": "corporation"
                                },
                                "corporation": {
                                    "id": 98000003,
                                    "name": "Kion Trading Inc.",
                                    "ticker": "KION"
                                },
                                "alliance": null
                            },
                            {
                                "id": 95000010,
                                "name": "Fresh Pilot",
                                "is_main": false,
                                "account": {
                                    "id": 3,
                                    "name": "Fresh Pilot"
                                },
                                "standing": {
                                    "value": -5,
                                    "source": "corporation"
                                },
                                "corporation": {
                                    "id": 98000003,
                                    "name": "Kion Trading Inc.",
                                    "ticker": "KION"
                                },
                                "alliance": null
                            },
                            {
                                "id": 95000002,
                                "name": "Tina Pilot",
                                "is_main": false,
                                "account": {
                                    "id": 2,
                                    "name": "Nicolas Pilot"
                                },
                                "standing": {
                                    "value": 0,
                                    "source": "character"
                                },
                                "corporation": {
                                    "id": 98000003,
                                    "name": "Kion Trading Inc.",
                                    "ticker": "KION"
                                },
                                "alliance": null
                            }
                        ]
                    },
                    {
                        "id": 98000004,
                        "name": "Red Corp",
                        "ticker": "REDC",
                        "standing": {
                            "value": -10,
                            "source": "corporation"
                        },
                        "characters": [
                            {
                                "id": 95000020,
                                "name": "Red Pilot",
                                "is_main": false,
                                "account": {
                                    "id": 4,
                                    "name": "Red Pilot"
                                },
                                "standing": {
                                    "value": -10,
                                    "source": "corporation"
                                },
                                "corporation": {
                                    "id": 98000004,
                                    "name": "Red Corp",
                                    "ticker": "REDC"
                                },
                                "alliance": {
                                    "id": 99000003,
                                    "name": "Red Alliance",
                                    "ticker": "RED"
                                }
                            }
                        ]
                    }
                ],
                "first_page_url": "https://bluebook.nicolaskion.dev/admin/pilots?view=corporations&page=1",
                "from": 1,
                "last_page": 1,
                "last_page_url": "https://bluebook.nicolaskion.dev/admin/pilots?view=corporations&page=1",
                "links": [
                    {
                        "url": null,
                        "label": "&laquo; Previous",
                        "page": null,
                        "active": false
                    },
                    {
                        "url": "https://bluebook.nicolaskion.dev/admin/pilots?view=corporations&page=1",
                        "label": "1",
                        "page": 1,
                        "active": true
                    },
                    {
                        "url": null,
                        "label": "Next &raquo;",
                        "page": null,
                        "active": false
                    }
                ],
                "next_page_url": null,
                "path": "https://bluebook.nicolaskion.dev/admin/pilots",
                "per_page": 20,
                "prev_page_url": null,
                "to": 5,
                "total": 5
            },
            "filters": {
                "search": "",
                "view": "corporations"
            }
        }),
        &own(&props),
    );
}

/// The corporation view finds a corporation by a member.
#[tokio::test]
async fn the_corporation_view_finds_a_corporation_by_a_member() {
    let app = TestApp::world().await;

    let props = page(
        &app,
        &app.sign_in_as(1, ADMIN).await,
        "/admin/pilots?view=corporations&search=tina",
    )
    .await;

    assert_json_eq(
        &json!({
            "characters": null,
            "groups": {
                "current_page": 1,
                "data": [
                    {
                        "id": 98000003,
                        "name": "Kion Trading Inc.",
                        "ticker": "KION",
                        "standing": {
                            "value": -5,
                            "source": "corporation"
                        },
                        "characters": [
                            {
                                "id": 2112000002,
                                "name": "admin alt",
                                "is_main": false,
                                "account": {
                                    "id": 1,
                                    "name": "Admin Pilot"
                                },
                                "standing": {
                                    "value": -5,
                                    "source": "corporation"
                                },
                                "corporation": {
                                    "id": 98000003,
                                    "name": "Kion Trading Inc.",
                                    "ticker": "KION"
                                },
                                "alliance": null
                            },
                            {
                                "id": 95000010,
                                "name": "Fresh Pilot",
                                "is_main": false,
                                "account": {
                                    "id": 3,
                                    "name": "Fresh Pilot"
                                },
                                "standing": {
                                    "value": -5,
                                    "source": "corporation"
                                },
                                "corporation": {
                                    "id": 98000003,
                                    "name": "Kion Trading Inc.",
                                    "ticker": "KION"
                                },
                                "alliance": null
                            },
                            {
                                "id": 95000002,
                                "name": "Tina Pilot",
                                "is_main": false,
                                "account": {
                                    "id": 2,
                                    "name": "Nicolas Pilot"
                                },
                                "standing": {
                                    "value": 0,
                                    "source": "character"
                                },
                                "corporation": {
                                    "id": 98000003,
                                    "name": "Kion Trading Inc.",
                                    "ticker": "KION"
                                },
                                "alliance": null
                            }
                        ]
                    }
                ],
                "first_page_url": "https://bluebook.nicolaskion.dev/admin/pilots?view=corporations&search=tina&page=1",
                "from": 1,
                "last_page": 1,
                "last_page_url": "https://bluebook.nicolaskion.dev/admin/pilots?view=corporations&search=tina&page=1",
                "links": [
                    {
                        "url": null,
                        "label": "&laquo; Previous",
                        "page": null,
                        "active": false
                    },
                    {
                        "url": "https://bluebook.nicolaskion.dev/admin/pilots?view=corporations&search=tina&page=1",
                        "label": "1",
                        "page": 1,
                        "active": true
                    },
                    {
                        "url": null,
                        "label": "Next &raquo;",
                        "page": null,
                        "active": false
                    }
                ],
                "next_page_url": null,
                "path": "https://bluebook.nicolaskion.dev/admin/pilots",
                "per_page": 20,
                "prev_page_url": null,
                "to": 1,
                "total": 1
            },
            "filters": {
                "search": "tina",
                "view": "corporations"
            }
        }),
        &own(&props),
    );
}

/// The corporation view searches names.
#[tokio::test]
async fn the_corporation_view_searches_names() {
    let app = TestApp::world().await;

    let props = page(
        &app,
        &app.sign_in_as(1, ADMIN).await,
        "/admin/pilots?view=corporations&search=kion",
    )
    .await;

    assert_json_eq(
        &json!({
            "characters": null,
            "groups": {
                "current_page": 1,
                "data": [
                    {
                        "id": 98000003,
                        "name": "Kion Trading Inc.",
                        "ticker": "KION",
                        "standing": {
                            "value": -5,
                            "source": "corporation"
                        },
                        "characters": [
                            {
                                "id": 2112000002,
                                "name": "admin alt",
                                "is_main": false,
                                "account": {
                                    "id": 1,
                                    "name": "Admin Pilot"
                                },
                                "standing": {
                                    "value": -5,
                                    "source": "corporation"
                                },
                                "corporation": {
                                    "id": 98000003,
                                    "name": "Kion Trading Inc.",
                                    "ticker": "KION"
                                },
                                "alliance": null
                            },
                            {
                                "id": 95000010,
                                "name": "Fresh Pilot",
                                "is_main": false,
                                "account": {
                                    "id": 3,
                                    "name": "Fresh Pilot"
                                },
                                "standing": {
                                    "value": -5,
                                    "source": "corporation"
                                },
                                "corporation": {
                                    "id": 98000003,
                                    "name": "Kion Trading Inc.",
                                    "ticker": "KION"
                                },
                                "alliance": null
                            },
                            {
                                "id": 95000002,
                                "name": "Tina Pilot",
                                "is_main": false,
                                "account": {
                                    "id": 2,
                                    "name": "Nicolas Pilot"
                                },
                                "standing": {
                                    "value": 0,
                                    "source": "character"
                                },
                                "corporation": {
                                    "id": 98000003,
                                    "name": "Kion Trading Inc.",
                                    "ticker": "KION"
                                },
                                "alliance": null
                            }
                        ]
                    }
                ],
                "first_page_url": "https://bluebook.nicolaskion.dev/admin/pilots?view=corporations&search=kion&page=1",
                "from": 1,
                "last_page": 1,
                "last_page_url": "https://bluebook.nicolaskion.dev/admin/pilots?view=corporations&search=kion&page=1",
                "links": [
                    {
                        "url": null,
                        "label": "&laquo; Previous",
                        "page": null,
                        "active": false
                    },
                    {
                        "url": "https://bluebook.nicolaskion.dev/admin/pilots?view=corporations&search=kion&page=1",
                        "label": "1",
                        "page": 1,
                        "active": true
                    },
                    {
                        "url": null,
                        "label": "Next &raquo;",
                        "page": null,
                        "active": false
                    }
                ],
                "next_page_url": null,
                "path": "https://bluebook.nicolaskion.dev/admin/pilots",
                "per_page": 20,
                "prev_page_url": null,
                "to": 1,
                "total": 1
            },
            "filters": {
                "search": "kion",
                "view": "corporations"
            }
        }),
        &own(&props),
    );
}

/// Pending requests, newest first, with the counts per status.
#[tokio::test]
async fn the_standing_requests_page() {
    let app = TestApp::world().await;

    let props = page(
        &app,
        &app.sign_in_as(1, ADMIN).await,
        "/admin/standing-requests",
    )
    .await;

    assert_json_eq(
        &json!({
            "standingRequests": {
                "current_page": 1,
                "data": [
                    {
                        "id": 2,
                        "status": "pending",
                        "created_at": "2026-06-25T13:30:00+00:00",
                        "subject": {
                            "type": "corporation",
                            "id": 98000003,
                            "name": "Kion Trading Inc."
                        },
                        "requested_by": "Tina Pilot",
                        "effective_standing": {
                            "standing": -5,
                            "source": "direct",
                            "via_type": "corporation",
                            "via_id": 98000003,
                            "via_name": "Kion Trading Inc."
                        }
                    },
                    {
                        "id": 5,
                        "status": "pending",
                        "created_at": "2026-06-25T13:15:00+00:00",
                        "subject": {
                            "type": "corporation",
                            "id": 98000005,
                            "name": "Écho Industries"
                        },
                        "requested_by": "Émile Pilot",
                        "effective_standing": null
                    },
                    {
                        "id": 1,
                        "status": "pending",
                        "created_at": "2026-06-25T13:00:00+00:00",
                        "subject": {
                            "type": "character",
                            "id": 95000010,
                            "name": "Fresh Pilot"
                        },
                        "requested_by": "Fresh Pilot",
                        "effective_standing": {
                            "standing": -5,
                            "source": "corporation",
                            "via_type": "corporation",
                            "via_id": 98000003,
                            "via_name": "Kion Trading Inc."
                        }
                    }
                ],
                "first_page_url": "https://bluebook.nicolaskion.dev/admin/standing-requests?page=1",
                "from": 1,
                "last_page": 1,
                "last_page_url": "https://bluebook.nicolaskion.dev/admin/standing-requests?page=1",
                "links": [
                    {
                        "url": null,
                        "label": "&laquo; Previous",
                        "page": null,
                        "active": false
                    },
                    {
                        "url": "https://bluebook.nicolaskion.dev/admin/standing-requests?page=1",
                        "label": "1",
                        "page": 1,
                        "active": true
                    },
                    {
                        "url": null,
                        "label": "Next &raquo;",
                        "page": null,
                        "active": false
                    }
                ],
                "next_page_url": null,
                "path": "https://bluebook.nicolaskion.dev/admin/standing-requests",
                "per_page": 25,
                "prev_page_url": null,
                "to": 3,
                "total": 3
            },
            "counts": {
                "pending": 3,
                "done": 1,
                "rejected": 1
            },
            "filters": {
                "status": "pending"
            }
        }),
        &own(&props),
    );
}

#[tokio::test]
async fn the_done_standing_requests() {
    let app = TestApp::world().await;

    let props = page(
        &app,
        &app.sign_in_as(1, ADMIN).await,
        "/admin/standing-requests?status=done",
    )
    .await;

    assert_json_eq(
        &json!({
            "standingRequests": {
                "current_page": 1,
                "data": [
                    {
                        "id": 4,
                        "status": "done",
                        "created_at": "2026-06-25T10:00:00+00:00",
                        "subject": {
                            "type": "character",
                            "id": 95000001,
                            "name": "Nicolas Pilot"
                        },
                        "requested_by": "Nicolas Pilot",
                        "effective_standing": {
                            "standing": 10,
                            "source": "corporation",
                            "via_type": "corporation",
                            "via_id": 98000002,
                            "via_name": "Blue Corp"
                        }
                    }
                ],
                "first_page_url": "https://bluebook.nicolaskion.dev/admin/standing-requests?status=done&page=1",
                "from": 1,
                "last_page": 1,
                "last_page_url": "https://bluebook.nicolaskion.dev/admin/standing-requests?status=done&page=1",
                "links": [
                    {
                        "url": null,
                        "label": "&laquo; Previous",
                        "page": null,
                        "active": false
                    },
                    {
                        "url": "https://bluebook.nicolaskion.dev/admin/standing-requests?status=done&page=1",
                        "label": "1",
                        "page": 1,
                        "active": true
                    },
                    {
                        "url": null,
                        "label": "Next &raquo;",
                        "page": null,
                        "active": false
                    }
                ],
                "next_page_url": null,
                "path": "https://bluebook.nicolaskion.dev/admin/standing-requests",
                "per_page": 25,
                "prev_page_url": null,
                "to": 1,
                "total": 1
            },
            "counts": {
                "pending": 3,
                "done": 1,
                "rejected": 1
            },
            "filters": {
                "status": "done"
            }
        }),
        &own(&props),
    );
}

#[tokio::test]
async fn the_rejected_standing_requests() {
    let app = TestApp::world().await;

    let props = page(
        &app,
        &app.sign_in_as(1, ADMIN).await,
        "/admin/standing-requests?status=rejected",
    )
    .await;

    assert_json_eq(
        &json!({
            "standingRequests": {
                "current_page": 1,
                "data": [
                    {
                        "id": 3,
                        "status": "rejected",
                        "created_at": "2026-06-25T12:00:00+00:00",
                        "subject": {
                            "type": "alliance",
                            "id": 99000003,
                            "name": "Red Alliance"
                        },
                        "requested_by": "Red Pilot",
                        "effective_standing": {
                            "standing": -10,
                            "source": "direct",
                            "via_type": "alliance",
                            "via_id": 99000003,
                            "via_name": "Red Alliance"
                        }
                    }
                ],
                "first_page_url": "https://bluebook.nicolaskion.dev/admin/standing-requests?status=rejected&page=1",
                "from": 1,
                "last_page": 1,
                "last_page_url": "https://bluebook.nicolaskion.dev/admin/standing-requests?status=rejected&page=1",
                "links": [
                    {
                        "url": null,
                        "label": "&laquo; Previous",
                        "page": null,
                        "active": false
                    },
                    {
                        "url": "https://bluebook.nicolaskion.dev/admin/standing-requests?status=rejected&page=1",
                        "label": "1",
                        "page": 1,
                        "active": true
                    },
                    {
                        "url": null,
                        "label": "Next &raquo;",
                        "page": null,
                        "active": false
                    }
                ],
                "next_page_url": null,
                "path": "https://bluebook.nicolaskion.dev/admin/standing-requests",
                "per_page": 25,
                "prev_page_url": null,
                "to": 1,
                "total": 1
            },
            "counts": {
                "pending": 3,
                "done": 1,
                "rejected": 1
            },
            "filters": {
                "status": "rejected"
            }
        }),
        &own(&props),
    );
}

/// An unknown status shows the pending requests.
#[tokio::test]
async fn an_unknown_status_shows_the_pending_requests() {
    let app = TestApp::world().await;

    let props = page(
        &app,
        &app.sign_in_as(1, ADMIN).await,
        "/admin/standing-requests?status=maybe",
    )
    .await;

    assert_json_eq(
        &json!({
            "standingRequests": {
                "current_page": 1,
                "data": [
                    {
                        "id": 2,
                        "status": "pending",
                        "created_at": "2026-06-25T13:30:00+00:00",
                        "subject": {
                            "type": "corporation",
                            "id": 98000003,
                            "name": "Kion Trading Inc."
                        },
                        "requested_by": "Tina Pilot",
                        "effective_standing": {
                            "standing": -5,
                            "source": "direct",
                            "via_type": "corporation",
                            "via_id": 98000003,
                            "via_name": "Kion Trading Inc."
                        }
                    },
                    {
                        "id": 5,
                        "status": "pending",
                        "created_at": "2026-06-25T13:15:00+00:00",
                        "subject": {
                            "type": "corporation",
                            "id": 98000005,
                            "name": "Écho Industries"
                        },
                        "requested_by": "Émile Pilot",
                        "effective_standing": null
                    },
                    {
                        "id": 1,
                        "status": "pending",
                        "created_at": "2026-06-25T13:00:00+00:00",
                        "subject": {
                            "type": "character",
                            "id": 95000010,
                            "name": "Fresh Pilot"
                        },
                        "requested_by": "Fresh Pilot",
                        "effective_standing": {
                            "standing": -5,
                            "source": "corporation",
                            "via_type": "corporation",
                            "via_id": 98000003,
                            "via_name": "Kion Trading Inc."
                        }
                    }
                ],
                "first_page_url": "https://bluebook.nicolaskion.dev/admin/standing-requests?status=maybe&page=1",
                "from": 1,
                "last_page": 1,
                "last_page_url": "https://bluebook.nicolaskion.dev/admin/standing-requests?status=maybe&page=1",
                "links": [
                    {
                        "url": null,
                        "label": "&laquo; Previous",
                        "page": null,
                        "active": false
                    },
                    {
                        "url": "https://bluebook.nicolaskion.dev/admin/standing-requests?status=maybe&page=1",
                        "label": "1",
                        "page": 1,
                        "active": true
                    },
                    {
                        "url": null,
                        "label": "Next &raquo;",
                        "page": null,
                        "active": false
                    }
                ],
                "next_page_url": null,
                "path": "https://bluebook.nicolaskion.dev/admin/standing-requests",
                "per_page": 25,
                "prev_page_url": null,
                "to": 3,
                "total": 3
            },
            "counts": {
                "pending": 3,
                "done": 1,
                "rejected": 1
            },
            "filters": {
                "status": "pending"
            }
        }),
        &own(&props),
    );
}

/// The source, what the admin character can make the source, the mail scope and the Discord settings.
#[tokio::test]
async fn the_admin_settings_page() {
    let app = TestApp::world().await;
    let webhook = app.discord.webhook();

    let props = page(&app, &app.sign_in_as(1, ADMIN).await, "/admin/settings").await;

    assert_json_eq(
        &json!({
            "source": {
                "type": "alliance",
                "entity_id": 99000002,
                "entity_name": "Home Alliance",
                "last_synced_at": "2026-06-25T13:58:00+00:00"
            },
            "sourceTypes": [
                {
                    "value": "character",
                    "label": "Character",
                    "entity_id": 2112000001,
                    "entity_name": "Admin Pilot",
                    "has_scope": true,
                    "available": true
                },
                {
                    "value": "corporation",
                    "label": "Corporation",
                    "entity_id": 98000001,
                    "entity_name": "Home Corp",
                    "has_scope": true,
                    "available": true
                },
                {
                    "value": "alliance",
                    "label": "Alliance",
                    "entity_id": 99000002,
                    "entity_name": "Home Alliance",
                    "has_scope": true,
                    "available": true
                }
            ],
            "adminCharacter": {
                "id": 2112000001,
                "name": "Admin Pilot",
                "has_mail_scope": true
            },
            "grantMailScopeUrl": "https://bluebook.nicolaskion.dev/eve?add_to_account=1&scopes=publicData%2Cesi-characters.read_contacts.v1%2Cesi-characters.write_contacts.v1%2Cesi-corporations.read_contacts.v1%2Cesi-alliances.read_contacts.v1%2Cesi-mail.send_mail.v1",
            "contactsCount": 10,
            "discordSettings": {
                "webhook_url": webhook,
                "role_id": "777"
            }
        }),
        &own(&props),
    );
}

/// An unknown view lists the characters; the search is trimmed.
#[tokio::test]
async fn the_pilots_page_falls_back_to_the_characters_view() {
    let app = TestApp::world().await;
    let browser = app.sign_in_as(1, ADMIN).await;
    let characters = page(&app, &browser, "/admin/pilots").await;

    let unknown = page(&app, &browser, "/admin/pilots?view=maybe").await;
    let searched = page(&app, &browser, "/admin/pilots?search=%20Pilot%20").await;

    assert_json_eq(&characters, &unknown);
    assert_json_eq(&characters["characters"], &searched["characters"]);
    assert_json_eq(
        &json!({ "search": "Pilot", "view": "characters" }),
        &searched["filters"],
    );
}

/// The group views page by 20, keeping the view and the search in the
/// links.
#[tokio::test]
async fn the_corporation_view_pages_by_twenty() {
    let app = TestApp::world().await;
    app.exec(
        "insert into corporations (id, name, ticker, created_at, updated_at)
         select 97000000 + n, 'Filler Corp ' || lpad(n::text, 2, '0'), 'FILL', now(), now()
         from generate_series(0, 21) n",
    )
    .await;
    app.exec(
        "insert into characters (id, name, corporation_id, user_id, should_sync, created_at,
            updated_at)
         select 96500000 + n, 'Filler Pilot ' || lpad(n::text, 2, '0'), 97000000 + n, 4, false,
            now(), now()
         from generate_series(0, 21) n",
    )
    .await;

    let props = page(
        &app,
        &app.sign_in_as(1, ADMIN).await,
        "/admin/pilots?view=corporations&search=Corp&page=2",
    )
    .await;

    let mut groups = props["groups"].clone();
    let names: Vec<&str> = props["groups"]["data"]
        .as_array()
        .expect("groups")
        .iter()
        .map(|group| group["name"].as_str().expect("a name"))
        .collect();
    assert_eq!(
        names,
        [
            "Filler Corp 19",
            "Filler Corp 20",
            "Filler Corp 21",
            "Home Corp",
            "Red Corp"
        ]
    );
    groups.as_object_mut().expect("a page").shift_remove("data");
    let link =
        |page: u32| format!("{APP_URL}/admin/pilots?view=corporations&search=Corp&page={page}");
    assert_json_eq(
        &json!({
            "current_page": 2,
            "first_page_url": link(1),
            "from": 21,
            "last_page": 2,
            "last_page_url": link(2),
            "links": [
                { "url": link(1), "label": "&laquo; Previous", "page": 1, "active": false },
                { "url": link(1), "label": "1", "page": 1, "active": false },
                { "url": link(2), "label": "2", "page": 2, "active": true },
                { "url": null, "label": "Next &raquo;", "page": null, "active": false }
            ],
            "next_page_url": null,
            "path": format!("{APP_URL}/admin/pilots"),
            "per_page": 20,
            "prev_page_url": link(1),
            "to": 25,
            "total": 25
        }),
        &groups,
    );
}

/// The requests page by 25, newest first.
#[tokio::test]
async fn the_standing_requests_page_by_twenty_five() {
    let app = TestApp::world().await;
    app.exec(
        "insert into standing_requests (id, subject_type, subject_id, requested_by_character_id,
            status, created_at, updated_at)
         select 1000 + n, 'character', 96000000 + n, 95000010, 'pending',
            '2026-06-25T05:40:00Z'::timestamptz - make_interval(secs => n), now()
         from generate_series(0, 27) n",
    )
    .await;

    let props = page(
        &app,
        &app.sign_in_as(1, ADMIN).await,
        "/admin/standing-requests?page=2",
    )
    .await;

    let mut requests = props["standingRequests"].clone();
    let ids: Vec<i64> = props["standingRequests"]["data"]
        .as_array()
        .expect("requests")
        .iter()
        .map(|request| request["id"].as_i64().expect("an id"))
        .collect();
    assert_eq!(ids, [1022, 1023, 1024, 1025, 1026, 1027]);
    requests
        .as_object_mut()
        .expect("a page")
        .shift_remove("data");
    let link = |page: u32| format!("{APP_URL}/admin/standing-requests?page={page}");
    assert_json_eq(
        &json!({
            "current_page": 2,
            "first_page_url": link(1),
            "from": 26,
            "last_page": 2,
            "last_page_url": link(2),
            "links": [
                { "url": link(1), "label": "&laquo; Previous", "page": 1, "active": false },
                { "url": link(1), "label": "1", "page": 1, "active": false },
                { "url": link(2), "label": "2", "page": 2, "active": true },
                { "url": null, "label": "Next &raquo;", "page": null, "active": false }
            ],
            "next_page_url": null,
            "path": format!("{APP_URL}/admin/standing-requests"),
            "per_page": 25,
            "prev_page_url": link(1),
            "to": 31,
            "total": 31
        }),
        &requests,
    );
    assert_json_eq(
        &json!({ "pending": 31, "done": 1, "rejected": 1 }),
        &props["counts"],
    );
}

/// With an alt active, the settings still offer the admin character's
/// entities and mail scope.
#[tokio::test]
async fn the_admin_settings_with_an_alt_active() {
    let app = TestApp::world().await;

    let alt = page(&app, &app.sign_in_as(1, ADMIN_ALT).await, "/admin/settings").await;

    let admin = page(&app, &app.sign_in_as(1, ADMIN).await, "/admin/settings").await;
    assert_json_eq(&own(&admin), &own(&alt));
    assert_json_eq(
        &json!({
            "id": 2112000002,
            "name": "admin alt",
            "corporation_id": 98000003,
            "alliance_id": null,
            "corporation": { "id": 98000003, "name": "Kion Trading Inc.", "ticker": "KION" },
            "alliance": null
        }),
        &alt["auth"]["active_character"],
    );
}

#[tokio::test]
async fn the_admin_settings_without_a_source_or_discord() {
    let app = TestApp::world().await;
    app.exec("delete from standings_sources").await;
    app.exec("delete from discord_settings").await;

    let props = page(&app, &app.sign_in_as(1, ADMIN).await, "/admin/settings").await;

    assert_json_eq(&Value::Null, &props["source"]);
    assert_json_eq(
        &json!({ "webhook_url": null, "role_id": null }),
        &props["discordSettings"],
    );
    assert_eq!(props["contactsCount"], 10);
}

/// Every page but the home page needs a login.
#[tokio::test]
async fn guests_are_sent_to_the_login_from_every_other_page() {
    let app = TestApp::world().await;

    for path in [
        "/dashboard",
        "/settings/appearance",
        "/admin",
        "/admin/standing-requests",
        "/admin/pilots",
        "/admin/settings",
    ] {
        let response = app.page(&Browser::guest(), path).await;
        response.assert_status(302);
        assert_eq!(response.location(), format!("{APP_URL}/eve"), "{path}");
    }
}

//! The dashboard (`DashboardController`): the source, its standings with
//! the redundancy flags (for those who may see them), the user's
//! characters and the entities they could request a standing for.

use std::collections::HashMap;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use super::AppState;
use super::account::{self, Account};
use super::errors;
use super::pages::{page_path, shared};
use super::viewer::Viewer;
use crate::format::php_float;
use crate::scopes::WRITE_CHARACTER_CONTACTS;
use crate::standings::{self, Source, key};

pub async fn page(
    State(state): State<AppState>,
    viewer: Viewer,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response, Response> {
    let account = account::page_account(&state, &viewer, &page_path(&uri)).await?;
    let mut props = shared(&state, &viewer, &headers, Some(&account)).await?;
    let source = Source::current(&state.pool)
        .await
        .map_err(errors::server_error)?;
    let can_view = can_view_standings(&state, &account, source.as_ref())
        .await
        .map_err(errors::server_error)?;
    props.insert(
        "showSetupWizard".to_owned(),
        json!(account.main_character_id.is_none() && account.setup_completed_at.is_none()),
    );
    let source_json = match &source {
        Some(source) => source
            .summary(&state.pool)
            .await
            .map_err(errors::server_error)?,
        None => Value::Null,
    };
    props.insert("source".to_owned(), source_json);
    props.insert("canViewStandings".to_owned(), json!(can_view));
    let standings = if can_view {
        standings(&state).await.map_err(errors::server_error)?
    } else {
        Value::Null
    };
    props.insert("standings".to_owned(), standings);
    props.insert(
        "characters".to_owned(),
        characters(&state, &account, source.as_ref())
            .await
            .map_err(errors::server_error)?,
    );
    props.insert(
        "requestableOptions".to_owned(),
        requestable_options(&state, &account, source.as_ref())
            .await
            .map_err(errors::server_error)?,
    );
    Ok(Json(props).into_response())
}

/// `User::canViewStandings`: admins always; others with a character the
/// source covers or that is blue directly or through its corporation or
/// alliance.
pub async fn can_view_standings(
    state: &AppState,
    account: &Account,
    source: Option<&Source>,
) -> sqlx::Result<bool> {
    if account.is_admin(state) {
        return Ok(true);
    }
    let Some(source) = source else {
        return Ok(false);
    };
    if account
        .characters
        .iter()
        .any(|character| source.covers(character.corporation_id, character.alliance_id))
    {
        return Ok(true);
    }
    let ids: Vec<i64> = account.characters.iter().map(|c| c.id).collect();
    let corporations: Vec<i64> = account
        .characters
        .iter()
        .filter_map(|c| c.corporation_id)
        .collect();
    let alliances: Vec<i64> = account
        .characters
        .iter()
        .filter_map(|c| c.alliance_id)
        .collect();
    sqlx::query_scalar(
        "select exists (select 1 from source_contacts where standing > 0 and (
            (contact_type = 'character' and contact_id = any($1))
            or (contact_type = 'corporation' and contact_id = any($2))
            or (contact_type = 'alliance' and contact_id = any($3))))",
    )
    .bind(ids)
    .bind(corporations)
    .bind(alliances)
    .fetch_one(&state.pool)
    .await
}

/// The source contacts, strongest first, each flagged with the parent
/// contact that makes it redundant: a character covered by its
/// corporation (first) or alliance, a corporation by its alliance.
async fn standings(state: &AppState) -> sqlx::Result<Value> {
    let contacts =
        standings::contacts(&state.pool, "c.standing desc, c.contact_type, c.id").await?;
    let by_key: HashMap<String, &standings::Contact> = contacts
        .iter()
        .map(|contact| (contact.key(), contact))
        .collect();
    let mut list = Vec::new();
    for contact in &contacts {
        let parents: Vec<(&str, Option<i64>)> = if !contact.known {
            Vec::new()
        } else {
            match contact.contact_type.as_str() {
                "character" => vec![
                    ("corporation", contact.parent_corporation_id),
                    ("alliance", contact.parent_alliance_id),
                ],
                "corporation" => vec![("alliance", contact.parent_alliance_id)],
                _ => Vec::new(),
            }
        };
        let mut redundant_via = Value::Null;
        for (kind, id) in parents {
            let Some(id) = id else { continue };
            if let Some(parent) = by_key.get(&key(kind, id))
                && parent.covers_standing(contact)
            {
                redundant_via = json!({
                    "contact_type": kind,
                    "contact_id": parent.contact_id,
                    "name": parent.name,
                });
                break;
            }
        }
        list.push(json!({
            "contact_id": contact.contact_id,
            "contact_type": contact.contact_type,
            "name": contact.name,
            "standing": php_float(contact.standing),
            "redundant_via": redundant_via,
        }));
    }
    Ok(Value::Array(list))
}

/// `CharacterSyncResource` of every character of the account.
async fn characters(
    state: &AppState,
    account: &Account,
    source: Option<&Source>,
) -> sqlx::Result<Value> {
    let facts: Vec<(i64, bool, i64)> = sqlx::query_as(
        "select c.id,
            exists (select 1 from esi_tokens t where t.character_id = c.id and $2 = any(t.scopes)),
            (select count(*) from character_synced_contacts s where s.character_id = c.id)
         from characters c where c.user_id = $1",
    )
    .bind(account.id)
    .bind(WRITE_CHARACTER_CONTACTS)
    .fetch_all(&state.pool)
    .await?;
    let facts: HashMap<i64, (bool, i64)> = facts
        .into_iter()
        .map(|(id, write, synced)| (id, (write, synced)))
        .collect();
    Ok(Value::Array(
        account
            .characters
            .iter()
            .map(|character| {
                let (write, synced) = facts.get(&character.id).copied().unwrap_or((false, 0));
                json!({
                    "id": character.id,
                    "name": character.name,
                    "is_main": account.main_character_id == Some(character.id),
                    "should_sync": character.should_sync,
                    "has_write_scope": write,
                    "inherits_source": source.is_some_and(|source| {
                        source.covers(character.corporation_id, character.alliance_id)
                    }),
                    "synced_contacts_count": synced,
                    "corporation": character.corporation.as_ref().map(account::Summary::json),
                    "alliance": character.alliance.as_ref().map(account::Summary::json),
                })
            })
            .collect(),
    ))
}

/// The characters, corporations and alliances the user could ask a
/// standing for, once each, each with a character to ask through and the
/// status of an earlier request. The source entity itself and characters
/// the source covers never are.
async fn requestable_options(
    state: &AppState,
    account: &Account,
    source: Option<&Source>,
) -> sqlx::Result<Value> {
    let Some(source) = source else {
        return Ok(json!([]));
    };
    let statuses: HashMap<String, String> = sqlx::query_as::<_, (String, i64, String)>(
        "select subject_type, subject_id, status from standing_requests",
    )
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(kind, id, status)| (key(&kind, id), status))
    .collect();
    let mut seen = Vec::new();
    let mut options = Vec::new();
    for character in &account.characters {
        if source.covers(character.corporation_id, character.alliance_id) {
            continue;
        }
        let entities = [
            ("character", Some(character.id), character.name.clone()),
            (
                "corporation",
                character.corporation_id,
                character.corporation.as_ref().and_then(|c| c.name.clone()),
            ),
            (
                "alliance",
                character.alliance_id,
                character.alliance.as_ref().and_then(|a| a.name.clone()),
            ),
        ];
        for (kind, id, name) in entities {
            let Some(id) = id else { continue };
            if kind == source.kind && id == source.entity_id {
                continue;
            }
            let entity_key = key(kind, id);
            if seen.contains(&entity_key) {
                continue;
            }
            options.push(json!({
                "type": kind,
                "id": id,
                "name": name,
                "via_character_id": character.id,
                "status": statuses.get(&entity_key),
            }));
            seen.push(entity_key);
        }
    }
    Ok(Value::Array(options))
}

//! The admin area behind `can:standings.admin`: the overview, the standing
//! requests, the pilots, the settings, and their forms (the source, the
//! sync now, resolving a request, the Discord settings).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::{Map, Value, json};

use super::AppState;
use super::account::{self, Account, CHARACTER_COLUMNS, CHARACTER_JOINS, Character, CharacterRow};
use super::answer::{Done, Errors, invalid_field};
use super::errors;
use super::input::{Input, query_pairs};
use super::pages::{page_path, shared};
use super::paginate::{Paginator, current_page};
use super::viewer::{self, Viewer};
use crate::auth::tokens::has_scope;
use crate::format::iso8601;
use crate::scopes::{ADMIN_SCOPES, SEND_MAIL, grant_url};
use crate::standings::effective::{RequestChain, Resolver};
use crate::standings::{self, Source};

const STATUSES: [&str; 3] = ["pending", "done", "rejected"];
const SOURCE_TYPES: [&str; 3] = ["character", "corporation", "alliance"];

/// An admin page: the account and the shared props.
async fn admin_page(
    state: &AppState,
    viewer: &Viewer,
    headers: &HeaderMap,
    uri: &Uri,
) -> Result<(Account, Map<String, Value>), Response> {
    let account = account::page_account(state, viewer, &page_path(uri)).await?;
    account::require_admin(state, &account)?;
    let props = shared(state, viewer, headers, Some(&account)).await?;
    Ok((account, props))
}

async fn source_summary(state: &AppState) -> Result<Value, Response> {
    match Source::current(&state.pool)
        .await
        .map_err(errors::server_error)?
    {
        Some(source) => source
            .summary(&state.pool)
            .await
            .map_err(errors::server_error),
        None => Ok(Value::Null),
    }
}

/// A standing request with its requester's names.
#[derive(sqlx::FromRow)]
struct RequestRow {
    id: i64,
    status: String,
    created_at: time::OffsetDateTime,
    subject_type: String,
    subject_id: i64,
    subject_name: Option<String>,
    requester: Option<String>,
    corporation_id: Option<i64>,
    alliance_id: Option<i64>,
}

/// A corporation or alliance of the pilot groups.
#[derive(sqlx::FromRow)]
struct GroupRow {
    id: i64,
    name: Option<String>,
    ticker: Option<String>,
    alliance_id: Option<i64>,
}

/// `StandingRequestResource` of the requests a query selects (with the
/// requester's names), with their effective standing.
async fn requests(
    state: &AppState,
    filter: &str,
    limit: i64,
    offset: i64,
    status: &str,
) -> sqlx::Result<Vec<Value>> {
    let rows: Vec<RequestRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select r.id, r.status, r.created_at, r.subject_type, r.subject_id,
            {} as subject_name, c.name as requester, c.corporation_id, c.alliance_id
         from standing_requests r
         join characters c on c.id = r.requested_by_character_id
         where {filter}
         order by r.created_at desc, r.id desc
         limit $2 offset $3",
        standings::SUBJECT_NAME_SQL
    )))
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;
    let resolver = Resolver::load(&state.pool).await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let effective = resolver.resolve(&RequestChain {
                subject_type: row.subject_type.clone(),
                subject_id: row.subject_id,
                requester_corporation_id: row.corporation_id,
                requester_alliance_id: row.alliance_id,
            });
            json!({
                "id": row.id,
                "status": row.status,
                "created_at": iso8601(row.created_at),
                "subject": { "type": row.subject_type, "id": row.subject_id, "name": row.subject_name },
                "requested_by": row.requester,
                "effective_standing": effective,
            })
        })
        .collect())
}

async fn count(state: &AppState, sql: &str) -> Result<i64, Response> {
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql.to_owned()))
        .fetch_one(&state.pool)
        .await
        .map_err(errors::server_error)
}

/// `/admin`: the source, the numbers and the five newest pending requests.
pub async fn overview(
    State(state): State<AppState>,
    viewer: Viewer,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response, Response> {
    let (_, mut props) = admin_page(&state, &viewer, &headers, &uri).await?;
    props.insert("source".to_owned(), source_summary(&state).await?);
    props.insert(
        "stats".to_owned(),
        json!({
            "pending_requests": count(&state, "select count(*) from standing_requests where status = 'pending'").await?,
            "pilots": count(&state, "select count(*) from users").await?,
            "syncing_characters": count(&state, "select count(*) from characters where user_id is not null and should_sync").await?,
            "source_contacts": count(&state, "select count(*) from source_contacts").await?,
        }),
    );
    let recent = requests(&state, "r.status = $1", 5, 0, "pending")
        .await
        .map_err(errors::server_error)?;
    props.insert("recentRequests".to_owned(), json!(recent));
    Ok(Json(props).into_response())
}

/// `/admin/standing-requests`: the requests of a status (pending unless a
/// known one is asked for), 25 a page, newest first, and the counts.
pub async fn standing_requests(
    State(state): State<AppState>,
    viewer: Viewer,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response, Response> {
    let (_, mut props) = admin_page(&state, &viewer, &headers, &uri).await?;
    let query = query_pairs(uri.query());
    let status = query
        .iter()
        .find(|(key, _)| key == "status")
        .map(|(_, value)| value.as_str())
        .filter(|value| STATUSES.contains(value))
        .unwrap_or("pending")
        .to_owned();
    let total: i64 = sqlx::query_scalar("select count(*) from standing_requests where status = $1")
        .bind(&status)
        .fetch_one(&state.pool)
        .await
        .map_err(errors::server_error)?;
    let paginator = Paginator {
        path: state.config.url("/admin/standing-requests"),
        query: &query,
        per_page: 25,
        current_page: current_page(&query),
        total,
    };
    let data = requests(&state, "r.status = $1", 25, paginator.offset(), &status)
        .await
        .map_err(errors::server_error)?;
    props.insert("standingRequests".to_owned(), paginator.json(data));
    let counts: (i64, i64, i64) = sqlx::query_as(
        "select count(*) filter (where status = 'pending'), count(*) filter (where status = 'done'),
            count(*) filter (where status = 'rejected')
         from standing_requests",
    )
    .fetch_one(&state.pool)
    .await
    .map_err(errors::server_error)?;
    props.insert(
        "counts".to_owned(),
        json!({ "pending": counts.0, "done": counts.1, "rejected": counts.2 }),
    );
    props.insert("filters".to_owned(), json!({ "status": status }));
    Ok(Json(props).into_response())
}

/// `characterRow`: a registered character, its account (named after its
/// main), its affiliations and its effective standing.
fn character_row(character: &Character, accounts: &AccountNames, resolver: &Resolver) -> Value {
    let (main, account_name) = character
        .user_id
        .and_then(|user| accounts.get(&user))
        .cloned()
        .unwrap_or((None, None));
    json!({
        "id": character.id,
        "name": character.name,
        "is_main": main == Some(character.id),
        "account": { "id": character.user_id, "name": account_name },
        "standing": resolver.for_character(character.id, character.corporation_id, character.alliance_id),
        "corporation": character.corporation.as_ref().map(account::Summary::json),
        "alliance": character.alliance.as_ref().map(account::Summary::json),
    })
}

/// Per user: the main character and the account's name (the main's
/// name, else the user's).
type AccountNames = std::collections::HashMap<i64, (Option<i64>, Option<String>)>;

async fn account_names(state: &AppState) -> sqlx::Result<AccountNames> {
    let rows: Vec<(i64, Option<i64>, Option<String>)> = sqlx::query_as(
        "select u.id, u.main_character_id, coalesce(m.name, u.name)
         from users u left join characters m on m.id = u.main_character_id",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, main, name)| (id, (main, name)))
        .collect())
}

async fn registered_characters(
    state: &AppState,
    filter: &str,
    bind: Option<i64>,
) -> sqlx::Result<Vec<Character>> {
    let rows: Vec<CharacterRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select {CHARACTER_COLUMNS} from characters c {CHARACTER_JOINS}
         where c.user_id is not null {filter}
         order by c.name nulls first, c.id"
    )))
    .bind(bind)
    .fetch_all(&state.pool)
    .await?;
    Ok(rows.into_iter().map(Character::from_row).collect())
}

/// `LIKE '%term%'` with the term taken literally (FIXES.md, "Percent and
/// underscore in the pilot search were wildcards").
fn like_pattern(term: &str) -> String {
    let escaped = term
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

/// `/admin/pilots`: every registered character (`view=characters`, the
/// default), or the corporations or alliances they are in, 20 a page,
/// searched by name, ticker or member name.
pub async fn pilots(
    State(state): State<AppState>,
    viewer: Viewer,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response, Response> {
    let (_, mut props) = admin_page(&state, &viewer, &headers, &uri).await?;
    let query = query_pairs(uri.query());
    let value = |name: &str| {
        query
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    };
    let search = value("search").trim().to_owned();
    let view = match value("view").as_str() {
        view @ ("corporations" | "alliances") => view.to_owned(),
        _ => "characters".to_owned(),
    };
    let resolver = Resolver::load(&state.pool)
        .await
        .map_err(errors::server_error)?;
    let accounts = account_names(&state).await.map_err(errors::server_error)?;

    if view == "characters" {
        let characters = registered_characters(&state, "and $1::bigint is null", None)
            .await
            .map_err(errors::server_error)?;
        props.insert(
            "characters".to_owned(),
            Value::Array(
                characters
                    .iter()
                    .map(|character| character_row(character, &accounts, &resolver))
                    .collect(),
            ),
        );
        props.insert("groups".to_owned(), Value::Null);
    } else {
        props.insert("characters".to_owned(), Value::Null);
        let groups = groups(&state, &view, &search, &query, &accounts, &resolver)
            .await
            .map_err(errors::server_error)?;
        props.insert("groups".to_owned(), groups);
    }
    props.insert(
        "filters".to_owned(),
        json!({ "search": search, "view": view }),
    );
    Ok(Json(props).into_response())
}

async fn groups(
    state: &AppState,
    view: &str,
    search: &str,
    query: &[(String, String)],
    accounts: &AccountNames,
    resolver: &Resolver,
) -> sqlx::Result<Value> {
    let (table, member_column) = if view == "corporations" {
        ("corporations", "corporation_id")
    } else {
        ("alliances", "alliance_id")
    };
    let alliance_column = if view == "corporations" {
        "g.alliance_id"
    } else {
        "null::bigint"
    };
    let filter = format!(
        "exists (select 1 from characters m where m.{member_column} = g.id and m.user_id is not null)
         and ($1 = '' or g.name ilike $2 escape '\\' or g.ticker ilike $2 escape '\\'
            or exists (select 1 from characters m where m.{member_column} = g.id
                and m.user_id is not null and m.name ilike $2 escape '\\'))"
    );
    let pattern = like_pattern(search);
    let total: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select count(*) from {table} g where {filter}"
    )))
    .bind(search)
    .bind(&pattern)
    .fetch_one(&state.pool)
    .await?;
    let paginator = Paginator {
        path: state.config.url("/admin/pilots"),
        query,
        per_page: 20,
        current_page: current_page(query),
        total,
    };
    let rows: Vec<GroupRow> =
        sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "select g.id, g.name, g.ticker, {alliance_column} as alliance_id from {table} g where {filter}
             order by g.name nulls first, g.id
             limit $3 offset $4"
        )))
        .bind(search)
        .bind(&pattern)
        .bind(paginator.per_page)
        .bind(paginator.offset())
        .fetch_all(&state.pool)
        .await?;
    let kind = if view == "corporations" {
        "corporation"
    } else {
        "alliance"
    };
    let mut data = Vec::new();
    for GroupRow {
        id,
        name,
        ticker,
        alliance_id,
    } in rows
    {
        let members =
            registered_characters(state, &format!("and c.{member_column} = $1"), Some(id)).await?;
        data.push(json!({
            "id": id,
            "name": name,
            "ticker": ticker,
            "standing": resolver.for_entity(kind, id, alliance_id),
            "characters": members
                .iter()
                .map(|character| character_row(character, accounts, resolver))
                .collect::<Vec<_>>(),
        }));
    }
    Ok(paginator.json(data))
}

/// The entity of a character a source type picks.
fn entity_of(character: &Character, kind: &str) -> (Option<i64>, Option<String>) {
    match kind {
        "character" => (Some(character.id), character.name.clone()),
        "corporation" => (
            character.corporation_id,
            character.corporation.as_ref().and_then(|c| c.name.clone()),
        ),
        _ => (
            character.alliance_id,
            character.alliance.as_ref().and_then(|a| a.name.clone()),
        ),
    }
}

fn label(kind: &str) -> String {
    let mut chars = kind.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

/// The admin character of the account to act through for a scope (and,
/// for a source type, the entity it gives): the account's admin
/// characters, the active one first, holding the scope (and having that
/// entity); else the active character when it is an admin; else the
/// first admin character. FIXES.md, "The source came from the active
/// character, not an admin character".
async fn admin_character(
    state: &AppState,
    account: &Account,
    scope: &str,
    kind: Option<&str>,
) -> Result<Character, Response> {
    let active = account.active_character(state).await?;
    let mut admins: Vec<&Character> = account
        .characters
        .iter()
        .filter(|character| state.config.is_admin_character(character.id))
        .collect();
    admins.sort_by_key(|character| (character.id != active.id, character.id));
    for character in &admins {
        if kind.is_some_and(|kind| entity_of(character, kind).0.is_none()) {
            continue;
        }
        if has_scope(&state.pool, character.id, scope)
            .await
            .map_err(errors::server_error)?
        {
            return Ok((*character).clone());
        }
    }
    Ok(admins.first().map_or(active, |first| (*first).clone()))
}

/// `/admin/settings`: the source and what the account's admin characters
/// could make the source, the mail scope, and the Discord settings.
pub async fn settings(
    State(state): State<AppState>,
    viewer: Viewer,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response, Response> {
    let (account, mut props) = admin_page(&state, &viewer, &headers, &uri).await?;
    props.insert("source".to_owned(), source_summary(&state).await?);
    let mut types = Vec::new();
    for kind in SOURCE_TYPES {
        let admin = admin_character(
            &state,
            &account,
            standings::required_scope(kind),
            Some(kind),
        )
        .await?;
        let (entity_id, entity_name) = entity_of(&admin, kind);
        let has_scope = has_scope(&state.pool, admin.id, standings::required_scope(kind))
            .await
            .map_err(errors::server_error)?;
        types.push(json!({
            "value": kind,
            "label": label(kind),
            "entity_id": entity_id,
            "entity_name": entity_name,
            "has_scope": has_scope,
            "available": entity_id.is_some() && has_scope,
        }));
    }
    props.insert("sourceTypes".to_owned(), json!(types));
    let admin = admin_character(&state, &account, SEND_MAIL, None).await?;
    let mail = has_scope(&state.pool, admin.id, SEND_MAIL)
        .await
        .map_err(errors::server_error)?;
    props.insert(
        "adminCharacter".to_owned(),
        json!({ "id": admin.id, "name": admin.name, "has_mail_scope": mail }),
    );
    props.insert(
        "grantMailScopeUrl".to_owned(),
        json!(grant_url(&state.config.app_url, &ADMIN_SCOPES)),
    );
    props.insert(
        "contactsCount".to_owned(),
        json!(count(&state, "select count(*) from source_contacts").await?),
    );
    let discord: Option<(Option<String>, Option<String>)> =
        sqlx::query_as("select webhook_url, role_id from discord_settings order by id limit 1")
            .fetch_optional(&state.pool)
            .await
            .map_err(errors::server_error)?;
    let (webhook_url, role_id) = discord.unwrap_or((None, None));
    props.insert(
        "discordSettings".to_owned(),
        json!({ "webhook_url": webhook_url, "role_id": role_id }),
    );
    Ok(Json(props).into_response())
}

/// An admin form: the account behind the login and the gate.
async fn admin_form(state: &AppState, viewer: &Viewer) -> Result<Account, Response> {
    let account = account::form_account(state, viewer).await?;
    account::require_admin(state, &account)?;
    Ok(account)
}

/// Flashes a message and stays on the page.
async fn back_with(
    state: &AppState,
    viewer: &Viewer,
    kind: &str,
    message: &str,
) -> Result<Response, Response> {
    let mut response = Done::back().into_response();
    viewer::flash(state, viewer, kind, message, &mut response)
        .await
        .map_err(errors::server_error)?;
    Ok(response)
}

/// `PUT /admin/standings`: the source becomes the admin character's
/// (`admin_character`) character, corporation or alliance; its standings are cleared when the
/// source changes.
pub async fn update_source(
    State(state): State<AppState>,
    viewer: Viewer,
    input: Input,
) -> Result<Response, Response> {
    let account = admin_form(&state, &viewer).await?;
    let mut errors_found = Errors::default();
    let kind = match input.get("type") {
        None => {
            errors_found.add("type", "The type field is required.");
            None
        }
        Some(value) => match value.as_str().filter(|kind| SOURCE_TYPES.contains(kind)) {
            Some(kind) => Some(kind.to_owned()),
            None => {
                errors_found.add("type", "The selected type is invalid.");
                None
            }
        },
    };
    errors_found.finish()?;
    let kind = kind.expect("a validated type");
    let character = admin_character(
        &state,
        &account,
        standings::required_scope(&kind),
        Some(&kind),
    )
    .await?;
    let Some(entity_id) = entity_of(&character, &kind).0 else {
        return Err(invalid_field(
            "type",
            &format!("Your character has no {kind} to use as a source."),
        ));
    };
    let current = Source::current(&state.pool)
        .await
        .map_err(errors::server_error)?;
    let changed = current
        .as_ref()
        .is_none_or(|source| source.kind != kind || source.entity_id != entity_id);
    let now = state.clock.now();
    let mut tx = state.pool.begin().await.map_err(errors::server_error)?;
    sqlx::query(
        "insert into standings_sources (type, entity_id, last_synced_at, created_at, updated_at)
         values ($1, $2, null, $3, $3)
         on conflict ((true)) do update set type = excluded.type, entity_id = excluded.entity_id,
            last_synced_at = null, unreadable_since = null, updated_at = excluded.updated_at",
    )
    .bind(&kind)
    .bind(entity_id)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(errors::server_error)?;
    if changed {
        sqlx::query("delete from source_contacts")
            .execute(&mut *tx)
            .await
            .map_err(errors::server_error)?;
    }
    tx.commit().await.map_err(errors::server_error)?;
    back_with(&state, &viewer, "success", "Standings source updated.").await
}

/// `POST /admin/standings/sync`: `standings:sync --force`, inside the
/// request, so the reloaded page shows the new time.
pub async fn sync_source(
    State(state): State<AppState>,
    viewer: Viewer,
) -> Result<Response, Response> {
    admin_form(&state, &viewer).await?;
    if crate::standings::sync::run(&state, true).await == 0 {
        back_with(&state, &viewer, "success", "Standings sync started.").await
    } else {
        // FIXES.md, ""Sync now" said the sync started when the source
        // could not be read".
        back_with(
            &state,
            &viewer,
            "error",
            "The standings source could not be read.",
        )
        .await
    }
}

/// `PUT /admin/standing-requests/{standingRequest}`.
pub async fn resolve_request(
    State(state): State<AppState>,
    viewer: Viewer,
    Path(id): Path<String>,
    input: Input,
) -> Result<Response, Response> {
    admin_form(&state, &viewer).await?;
    let not_found = || errors::model_not_found("StandingRequest", &id);
    let request_id: i64 = id.parse().map_err(|_| not_found())?;
    let exists: bool =
        sqlx::query_scalar("select exists (select 1 from standing_requests where id = $1)")
            .bind(request_id)
            .fetch_one(&state.pool)
            .await
            .map_err(errors::server_error)?;
    if !exists {
        return Err(not_found());
    }
    let mut errors_found = Errors::default();
    let status = match input.get("status") {
        None => {
            errors_found.add("status", "The status field is required.");
            None
        }
        Some(value) => match value.as_str().filter(|status| STATUSES.contains(status)) {
            Some(status) => Some(status.to_owned()),
            None => {
                errors_found.add("status", "The selected status is invalid.");
                None
            }
        },
    };
    errors_found.finish()?;
    sqlx::query("update standing_requests set status = $2, updated_at = $3 where id = $1")
        .bind(request_id)
        .bind(status)
        .bind(state.clock.now())
        .execute(&state.pool)
        .await
        .map_err(errors::server_error)?;
    back_with(&state, &viewer, "success", "Standing request updated.").await
}

/// Laravel's `url` rule, as far as a webhook needs it: an absolute URL
/// with a host.
fn is_url(value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|url| url.has_host())
}

/// `PUT /admin/discord-settings`.
pub async fn update_discord(
    State(state): State<AppState>,
    viewer: Viewer,
    input: Input,
) -> Result<Response, Response> {
    admin_form(&state, &viewer).await?;
    let mut errors_found = Errors::default();
    let webhook = input.get("webhook_url");
    if let Some(value) = webhook {
        match value.as_str() {
            Some(text) if is_url(text) => {
                if !text.starts_with("https://") {
                    errors_found.add(
                        "webhook_url",
                        "The webhook url field must start with one of the following: https://.",
                    );
                }
            }
            _ => errors_found.add("webhook_url", "The webhook url field must be a valid URL."),
        }
    }
    let role = input.get("role_id");
    if let Some(value) = role {
        match value.as_str() {
            Some(text) => {
                if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
                    errors_found.add("role_id", "The role id field format is invalid.");
                }
            }
            None => errors_found.add("role_id", "The role id field must be a string."),
        }
    }
    errors_found.finish()?;
    let now = state.clock.now();
    sqlx::query(
        "insert into discord_settings (webhook_url, role_id, created_at, updated_at)
         values ($1, $2, $3, $3)
         on conflict ((true)) do update set webhook_url = excluded.webhook_url,
            role_id = excluded.role_id, updated_at = excluded.updated_at",
    )
    .bind(webhook.and_then(Value::as_str))
    .bind(role.and_then(Value::as_str))
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(errors::server_error)?;
    back_with(&state, &viewer, "success", "Discord settings updated.").await
}

//! The props of the SvelteKit pages, the Inertia props of the Laravel
//! pages (`/api/pages/*`): the shared props of `HandleInertiaRequests`
//! (`name`, `auth`, `sidebarOpen`, `flash`) merged with each page's own.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::{Map, Value, json};

use super::AppState;
use super::account::{self, Account};
use super::errors;
use super::viewer::Viewer;
use crate::auth::session;
use crate::auth::tokens::usable_sql;
use crate::scopes::{ADMIN_SCOPES, SYNC_SCOPES, WRITE_CHARACTER_CONTACTS, grant_url};
use crate::standings::{Source, source};

/// The page a props request is for: `/api/pages/dashboard?x` is
/// `/dashboard?x`, the home page `/`.
pub fn page_path(uri: &Uri) -> String {
    // Inside the `/api` router the path has lost its `/api`.
    let path = uri.path();
    let path = path
        .strip_prefix("/api/pages")
        .or_else(|| path.strip_prefix("/pages"))
        .unwrap_or(path);
    let path = if path == "/home" { "/" } else { path };
    match uri.query() {
        Some(query) => format!("{path}?{query}"),
        None => path.to_owned(),
    }
}

/// `sidebarOpen`: open unless the `sidebar_state` cookie says otherwise.
fn sidebar_open(headers: &HeaderMap) -> bool {
    session::cookie_value(headers, "sidebar_state").is_none_or(|value| value == "true")
}

/// The flash message of the session, taken.
async fn flash(state: &AppState, viewer: &Viewer) -> Result<Value, Response> {
    let mut flash = json!({ "success": null, "error": null, "info": null });
    if let Some(id) = viewer.session_id()
        && let Some(message) = session::pull(&state.pool, id, "flash")
            .await
            .map_err(errors::server_error)?
        && let (Some(kind), Some(text)) = (message["kind"].as_str(), message["message"].as_str())
    {
        flash[kind] = json!(text);
    }
    Ok(flash)
}

/// The shared `auth` of a signed-in user.
async fn auth(state: &AppState, account: &Account) -> Result<Value, Response> {
    let active = account.active_character(state).await?;
    let is_admin = account.is_admin(state);
    // No admin character can read the source, or the last read was
    // refused (403, a missing in-game role).
    let source_unreadable = if is_admin {
        match Source::current(&state.pool)
            .await
            .map_err(errors::server_error)?
        {
            Some(source) => {
                source.unreadable_since.is_some()
                    || source::reader(state, &source)
                        .await
                        .map_err(errors::server_error)?
                        .is_none()
            }
            None => false,
        }
    } else {
        false
    };
    // Only characters that opted into syncing need the contact scopes; a
    // token failing for three days counts as lapsed.
    let reauth: Vec<(i64, Option<String>)> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select c.id, c.name from characters c
         where c.user_id = $1 and c.should_sync
            and not exists (select 1 from esi_tokens t
                where t.character_id = c.id and $2 = any(t.scopes) and {})
         order by c.id",
        usable_sql("$3")
    )))
    .bind(account.id)
    .bind(WRITE_CHARACTER_CONTACTS)
    .bind(state.clock.now())
    .fetch_all(&state.pool)
    .await
    .map_err(errors::server_error)?;
    Ok(json!({
        "user": { "id": account.id, "name": account.name },
        "is_admin": is_admin,
        "source_unreadable": source_unreadable,
        "reauth_characters": reauth
            .into_iter()
            .map(|(id, name)| json!({ "id": id, "name": name }))
            .collect::<Vec<_>>(),
        "sync_scopes_url": grant_url(&state.config.app_url, &SYNC_SCOPES),
        "admin_scopes_url": is_admin.then(|| grant_url(&state.config.app_url, &ADMIN_SCOPES)),
        "active_character": active.json(),
        "characters": account.characters.iter().map(account::Character::json).collect::<Vec<_>>(),
    }))
}

/// The shared props.
pub async fn shared(
    state: &AppState,
    viewer: &Viewer,
    headers: &HeaderMap,
    account: Option<&Account>,
) -> Result<Map<String, Value>, Response> {
    let auth = match account {
        Some(account) => auth(state, account).await?,
        None => json!({ "user": null }),
    };
    let mut props = Map::new();
    props.insert("name".to_owned(), json!(state.config.app_name));
    props.insert("auth".to_owned(), auth);
    props.insert("sidebarOpen".to_owned(), json!(sidebar_open(headers)));
    props.insert("flash".to_owned(), flash(state, viewer).await?);
    Ok(props)
}

/// The landing page (Welcome), and whether the demo login is on.
pub async fn home(
    State(state): State<AppState>,
    viewer: Viewer,
    headers: HeaderMap,
) -> Result<Response, Response> {
    let account = Account::load(&state, &viewer)
        .await
        .map_err(errors::server_error)?;
    let mut props = shared(&state, &viewer, &headers, account.as_ref()).await?;
    if state.config.demo.enabled {
        props.insert("demoLoginEnabled".to_owned(), json!(true));
    }
    Ok(Json(props).into_response())
}

/// The appearance settings: the shared props only.
pub async fn appearance(
    State(state): State<AppState>,
    viewer: Viewer,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response, Response> {
    let account = account::page_account(&state, &viewer, &page_path(&uri)).await?;
    let props = shared(&state, &viewer, &headers, Some(&account)).await?;
    Ok(Json(props).into_response())
}

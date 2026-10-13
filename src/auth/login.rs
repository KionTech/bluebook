//! The EVE login (`EveController`, `EsiAuthService`), the logout and the
//! local demo login.
//!
//! `/eve` sends the browser to EVE SSO with the scopes a grant link asks
//! for and a random state, kept in the session (a guest gets one for it).
//! `?add_to_account=1` remembers the signed-in user for the callback, until
//! the session ends, as Laravel's session did. The callback checks the
//! state, trades the code, reads the character's corporation and alliance
//! from ESI, stores the character and its token (when it carries scopes),
//! and signs in: the account it is added to, the account it belongs to
//! when the EVE account is the same, or a new one.

use std::collections::HashMap;

use axum::extract::{RawQuery, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Map, Value, json};
use time::Duration;

use super::session;
use super::sso::Login;
use crate::entities;
use crate::scopes;
use crate::server::AppState;
use crate::server::account;
use crate::server::answer::{self, Done};
use crate::server::errors;
use crate::server::input::query_pairs;
use crate::server::viewer::{self, Viewer};

const STATE_KEY: &str = "sso_state";
const LOGIN_FAILED: &str =
    "Unable to retrieve user information from EVE Online. Please try again later.";

/// `GET /eve[?add_to_account=1][&scopes=a,b]`.
pub async fn redirect(
    State(state): State<AppState>,
    viewer: Viewer,
    RawQuery(query): RawQuery,
) -> Result<Response, Response> {
    let query: HashMap<String, String> = query_pairs(query.as_deref()).into_iter().collect();
    let requested: Vec<String> = match query.get("scopes").filter(|scopes| !scopes.is_empty()) {
        Some(list) => list.split(',').map(str::to_owned).collect(),
        None => Vec::new(),
    };
    // FIXES.md, "An unknown scope in the login link was a server error".
    if requested.iter().any(|scope| !scopes::is_known(scope)) {
        return Err(errors::http_error(StatusCode::BAD_REQUEST, "Bad Request"));
    }
    let login_state = session::random_string(40);
    let location = state.sso.authorize_url(&requested, &login_state);
    let mut response = answer::redirect(&location);
    let add_to_account = query
        .get("add_to_account")
        .is_some_and(|value| !value.is_empty() && value != "0");
    match viewer.session_id() {
        Some(id) => {
            session::put(&state.pool, id, STATE_KEY, json!(login_state))
                .await
                .map_err(errors::server_error)?;
            if add_to_account {
                session::put(&state.pool, id, "add_to_account", json!(viewer.user_id()))
                    .await
                    .map_err(errors::server_error)?;
            }
        }
        None => {
            viewer::put(
                &state,
                &viewer,
                STATE_KEY,
                json!(login_state),
                &mut response,
            )
            .await
            .map_err(errors::server_error)?;
        }
    }
    Ok(response)
}

/// The home page with the login's error.
async fn failed(state: &AppState, viewer: &Viewer) -> Result<Response, Response> {
    let mut response = answer::redirect(&state.config.app_url);
    viewer::flash(state, viewer, "error", LOGIN_FAILED, &mut response)
        .await
        .map_err(errors::server_error)?;
    Ok(response)
}

/// `GET /eve/callback?code=...&state=...`.
pub async fn callback(
    State(state): State<AppState>,
    viewer: Viewer,
    RawQuery(query): RawQuery,
) -> Result<Response, Response> {
    let query: HashMap<String, String> = query_pairs(query.as_deref()).into_iter().collect();
    let expected = match viewer.session_id() {
        Some(id) => session::pull(&state.pool, id, STATE_KEY)
            .await
            .map_err(errors::server_error)?,
        None => None,
    };
    let (Some(Value::String(expected)), Some(given), Some(code)) =
        (expected, query.get("state"), query.get("code"))
    else {
        return failed(&state, &viewer).await;
    };
    if *given != expected {
        return failed(&state, &viewer).await;
    }
    let login = match state.sso.login(code).await {
        Ok(login) => login,
        Err(error) => {
            tracing::warn!(%error, "EVE login failed");
            return failed(&state, &viewer).await;
        }
    };
    let affiliation = match state.esi.affiliations(&[login.character_id]).await {
        Ok(mut affiliations) if !affiliations.is_empty() => affiliations.remove(0),
        _ => return failed(&state, &viewer).await,
    };
    entities::ensure_affiliation(&state, &affiliation)
        .await
        .map_err(errors::server_error)?;
    let now = state.clock.now();
    sqlx::query(
        "insert into characters (id, name, corporation_id, alliance_id, created_at, updated_at)
         values ($1, $2, $3, $4, $5, $5)
         on conflict (id) do update set name = excluded.name, corporation_id = excluded.corporation_id,
            alliance_id = excluded.alliance_id, updated_at = excluded.updated_at",
    )
    .bind(login.character_id)
    .bind(&login.character_name)
    .bind(affiliation.corporation_id)
    .bind(affiliation.alliance_id.filter(|id| *id != 0))
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(errors::server_error)?;
    store_token(&state, &login)
        .await
        .map_err(errors::server_error)?;

    let add_to = viewer
        .session
        .as_ref()
        .and_then(|session| session.get_i64("add_to_account"));
    let user_id = match add_to {
        Some(user_id) => {
            let exists: bool =
                sqlx::query_scalar("select exists (select 1 from users where id = $1)")
                    .bind(user_id)
                    .fetch_one(&state.pool)
                    .await
                    .map_err(errors::server_error)?;
            if !exists {
                return Err(errors::model_not_found("User", &user_id.to_string()));
            }
            transfer(&state, login.character_id, user_id)
                .await
                .map_err(errors::server_error)?;
            user_id
        }
        None => owner(&state, &login).await.map_err(errors::server_error)?,
    };

    let (target, message) = if add_to.is_some() {
        (
            state.config.url("/dashboard"),
            "Your character has been added to your account.".to_owned(),
        )
    } else {
        let intended = match viewer.session_id() {
            Some(id) => session::pull(&state.pool, id, "intended")
                .await
                .map_err(errors::server_error)?
                .and_then(|value| value.as_str().map(str::to_owned)),
            None => None,
        };
        (
            state
                .config
                .url(intended.as_deref().unwrap_or("/dashboard")),
            format!("Welcome back, {}!", login.character_name),
        )
    };
    let mut response = answer::redirect(&target);
    let session_id = sign_in(&state, &viewer, user_id, login.character_id, &mut response)
        .await
        .map_err(errors::server_error)?;
    session::put(
        &state.pool,
        &session_id,
        "flash",
        viewer::flash_value("success", &message),
    )
    .await
    .map_err(errors::server_error)?;
    Ok(response)
}

/// `createEsiToken`: the login's token with the scopes the app knows. A
/// login without scopes stores nothing (FIXES.md, "Every scopeless login
/// stored a token that was never used").
async fn store_token(state: &AppState, login: &Login) -> sqlx::Result<()> {
    let known: Vec<String> = login
        .scopes
        .iter()
        .filter(|scope| scopes::is_known(scope))
        .cloned()
        .collect();
    if known.is_empty() {
        return Ok(());
    }
    let now = state.clock.now();
    sqlx::query(
        "insert into esi_tokens (character_id, access_token, token_type, refresh_token,
            character_owner_hash, scopes, expires_at, created_at, updated_at)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $8)",
    )
    .bind(login.character_id)
    .bind(&login.access_token)
    .bind(&login.token_type)
    .bind(&login.refresh_token)
    .bind(&login.owner_hash)
    .bind(known)
    .bind(now + Duration::seconds(login.expires_in))
    .bind(now)
    .execute(&state.pool)
    .await?;
    Ok(())
}

/// The account a login signs into without `add_to_account`: the
/// character's when the EVE account (owner hash) is the same, else a new
/// one named after the character.
async fn owner(state: &AppState, login: &Login) -> sqlx::Result<i64> {
    let current: Option<(Option<i64>, Option<String>)> =
        sqlx::query_as("select user_id, character_owner_hash from characters where id = $1")
            .bind(login.character_id)
            .fetch_optional(&state.pool)
            .await?;
    if let Some((Some(user_id), Some(hash))) = &current
        && *hash == login.owner_hash
    {
        return Ok(*user_id);
    }
    let now = state.clock.now();
    let user_id: i64 = sqlx::query_scalar(
        "insert into users (name, created_at, updated_at) values ($1, $2, $2) returning id",
    )
    .bind(&login.character_name)
    .bind(now)
    .fetch_one(&state.pool)
    .await?;
    sqlx::query("update characters set character_owner_hash = $2 where id = $1")
        .bind(login.character_id)
        .bind(&login.owner_hash)
        .execute(&state.pool)
        .await?;
    transfer(state, login.character_id, user_id).await?;
    Ok(user_id)
}

/// Moves a character to an account. The account it came from loses a
/// main selection pointing at it, and goes when it is left without
/// characters.
async fn transfer(state: &AppState, character_id: i64, user_id: i64) -> sqlx::Result<()> {
    let previous: Option<i64> = sqlx::query_scalar("select user_id from characters where id = $1")
        .bind(character_id)
        .fetch_one(&state.pool)
        .await?;
    let now = state.clock.now();
    sqlx::query("update characters set user_id = $2, updated_at = $3 where id = $1")
        .bind(character_id)
        .bind(user_id)
        .bind(now)
        .execute(&state.pool)
        .await?;
    let Some(previous) = previous.filter(|previous| *previous != user_id) else {
        return Ok(());
    };
    sqlx::query(
        "update users set main_character_id = null, updated_at = $3 where id = $1 and main_character_id = $2",
    )
    .bind(previous)
    .bind(character_id)
    .bind(now)
    .execute(&state.pool)
    .await?;
    sqlx::query(
        "delete from users u where u.id = $1 and not exists (select 1 from characters c where c.user_id = u.id)",
    )
    .bind(previous)
    .execute(&state.pool)
    .await?;
    Ok(())
}

/// Signs a user in with a new session token, carrying over what the old
/// session held (Laravel's `Auth::login` migrates the session), with the
/// character active.
async fn sign_in(
    state: &AppState,
    viewer: &Viewer,
    user_id: i64,
    character_id: i64,
    response: &mut Response,
) -> sqlx::Result<String> {
    let mut data: Map<String, Value> = viewer
        .session
        .as_ref()
        .map(|session| session.data.0.clone())
        .unwrap_or_default();
    data.insert("active_character_id".to_owned(), json!(character_id));
    if let Some(id) = viewer.session_id() {
        session::end(&state.pool, id).await?;
    }
    let token = session::start(&state.pool, Some(user_id), data, state.clock.now()).await?;
    let id = session::hash_token(&token);
    let cookie = session::cookie(
        token,
        state.config.session_lifetime_days,
        state.config.secure_cookies(),
    );
    if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    Ok(id)
}

/// `POST /logout`: the session ends, the browser goes home.
pub async fn logout(State(state): State<AppState>, viewer: Viewer) -> Result<Response, Response> {
    account::form_account(&state, &viewer).await?;
    if let Some(id) = viewer.session_id() {
        session::end(&state.pool, id)
            .await
            .map_err(errors::server_error)?;
    }
    Ok(Done::to("/")
        .cookie(session::removal(state.config.secure_cookies()))
        .into_response())
}

/// `GET /demo-login[?user=<id>]`: signs in as the demo pilot (or a given
/// user) without EVE SSO when DEMO_ENABLED (FIXES.md, "The demo login");
/// not found otherwise.
pub async fn demo(
    State(state): State<AppState>,
    viewer: Viewer,
    RawQuery(query): RawQuery,
) -> Result<Response, Response> {
    if !state.config.demo.enabled {
        return Err(errors::http_error(
            StatusCode::NOT_FOUND,
            "The route demo-login could not be found.",
        ));
    }
    let query: HashMap<String, String> = query_pairs(query.as_deref()).into_iter().collect();
    let now = state.clock.now();
    let user_id = match query.get("user").and_then(|id| id.parse::<i64>().ok()) {
        Some(id) => id,
        None => crate::demo::pilot(&state.pool, now, state.config.demo.character_id)
            .await
            .map_err(errors::server_error)?,
    };
    let character: Option<i64> =
        sqlx::query_scalar("select id from characters where user_id = $1 order by id limit 1")
            .bind(user_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(errors::server_error)?;
    let Some(character) = character else {
        return Err(errors::http_error(StatusCode::NOT_FOUND, "Not Found"));
    };
    let mut response = answer::redirect(&state.config.url("/dashboard"));
    sign_in(&state, &viewer, user_id, character, &mut response)
        .await
        .map_err(errors::server_error)?;
    Ok(response)
}

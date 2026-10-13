//! The signed-in user of a request with its characters, the active
//! character in the session, and the gates of the pages and forms (signed
//! in, admin, own character).

use axum::http::StatusCode;
use axum::response::Response;
use serde_json::{Value, json};
use time::OffsetDateTime;

use super::AppState;
use super::answer;
use super::errors;
use super::viewer::{self, Viewer};
use crate::auth::session;

/// A corporation or alliance as the characters carry it.
#[derive(Debug, Clone)]
pub struct Summary {
    pub id: i64,
    pub name: Option<String>,
    pub ticker: Option<String>,
}

impl Summary {
    pub fn json(&self) -> Value {
        json!({ "id": self.id, "name": self.name, "ticker": self.ticker })
    }
}

/// A character with its corporation and alliance.
#[derive(Debug, Clone)]
pub struct Character {
    pub id: i64,
    pub name: Option<String>,
    pub corporation_id: Option<i64>,
    pub alliance_id: Option<i64>,
    pub user_id: Option<i64>,
    pub should_sync: bool,
    pub corporation: Option<Summary>,
    pub alliance: Option<Summary>,
}

pub type CharacterRow = (
    i64,
    Option<String>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    bool,
    Option<i64>,
    Option<String>,
    Option<String>,
    Option<i64>,
    Option<String>,
    Option<String>,
);

/// The select list of `Character::from_row`, `c` the characters table.
pub const CHARACTER_COLUMNS: &str = "c.id, c.name, c.corporation_id, c.alliance_id, c.user_id,
    c.should_sync, co.id, co.name, co.ticker, al.id, al.name, al.ticker";

/// The joins of `CHARACTER_COLUMNS`.
pub const CHARACTER_JOINS: &str = "left join corporations co on co.id = c.corporation_id
    left join alliances al on al.id = c.alliance_id";

impl Character {
    pub fn from_row(row: CharacterRow) -> Self {
        let (
            id,
            name,
            corporation_id,
            alliance_id,
            user_id,
            should_sync,
            co_id,
            co_name,
            co_ticker,
            al_id,
            al_name,
            al_ticker,
        ) = row;
        Self {
            id,
            name,
            corporation_id,
            alliance_id,
            user_id,
            should_sync,
            // Only an entity that is stored locally.
            corporation: co_id.map(|id| Summary {
                id,
                name: co_name,
                ticker: co_ticker,
            }),
            alliance: al_id.map(|id| Summary {
                id,
                name: al_name,
                ticker: al_ticker,
            }),
        }
    }

    /// The character with its corporation and alliance.
    pub fn json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "corporation_id": self.corporation_id,
            "alliance_id": self.alliance_id,
            "corporation": self.corporation.as_ref().map(Summary::json),
            "alliance": self.alliance.as_ref().map(Summary::json),
        })
    }
}

/// A character by id, with its corporation and alliance.
pub async fn character(state: &AppState, id: i64) -> sqlx::Result<Option<Character>> {
    let row: Option<CharacterRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select {CHARACTER_COLUMNS} from characters c {CHARACTER_JOINS} where c.id = $1"
    )))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;
    Ok(row.map(Character::from_row))
}

/// The signed-in user and its characters, by id.
#[derive(Debug, Clone)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub main_character_id: Option<i64>,
    pub setup_completed_at: Option<OffsetDateTime>,
    pub characters: Vec<Character>,
    pub session_id: String,
    /// The active character the session names.
    pub session_character: Option<i64>,
}

impl Account {
    /// The viewer's account, if signed in.
    pub async fn load(state: &AppState, viewer: &Viewer) -> sqlx::Result<Option<Account>> {
        let (Some(user_id), Some(session)) = (viewer.user_id(), viewer.session.as_ref()) else {
            return Ok(None);
        };
        let user: Option<(i64, String, Option<i64>, Option<OffsetDateTime>)> = sqlx::query_as(
            "select id, name, main_character_id, setup_completed_at from users where id = $1",
        )
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?;
        let Some((id, name, main_character_id, setup_completed_at)) = user else {
            return Ok(None);
        };
        let rows: Vec<CharacterRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "select {CHARACTER_COLUMNS} from characters c {CHARACTER_JOINS}
             where c.user_id = $1 order by c.id"
        )))
        .bind(id)
        .fetch_all(&state.pool)
        .await?;
        Ok(Some(Account {
            id,
            name,
            main_character_id,
            setup_completed_at,
            characters: rows.into_iter().map(Character::from_row).collect(),
            session_id: session.id.clone(),
            session_character: session.get_i64("active_character_id"),
        }))
    }

    pub fn is_admin(&self, state: &AppState) -> bool {
        self.characters
            .iter()
            .any(|character| state.config.is_admin_character(character.id))
    }

    pub fn owns(&self, character_id: i64) -> bool {
        self.characters
            .iter()
            .any(|character| character.id == character_id)
    }

    /// The session's character when it is still on the account, else the
    /// first (now stored in the session). Without characters the user is
    /// signed out with a 403.
    pub async fn active_character(&self, state: &AppState) -> Result<Character, Response> {
        if let Some(character) = self
            .session_character
            .and_then(|id| self.characters.iter().find(|character| character.id == id))
        {
            return Ok(character.clone());
        }
        let Some(first) = self.characters.first() else {
            session::end(&state.pool, &self.session_id)
                .await
                .map_err(errors::server_error)?;
            return Err(errors::http_error(
                StatusCode::FORBIDDEN,
                "No characters found. Please log in again.",
            ));
        };
        set_active(state, &self.session_id, first.id)
            .await
            .map_err(errors::server_error)?;
        Ok(first.clone())
    }
}

/// Stores the active character in a session.
pub async fn set_active(state: &AppState, session_id: &str, character_id: i64) -> sqlx::Result<()> {
    session::put(
        &state.pool,
        session_id,
        "active_character_id",
        json!(character_id),
    )
    .await
}

/// A page behind the login: the account, or the redirect to the login
/// with the page remembered for after it.
pub async fn page_account(
    state: &AppState,
    viewer: &Viewer,
    page: &str,
) -> Result<Account, Response> {
    if let Some(account) = Account::load(state, viewer)
        .await
        .map_err(errors::server_error)?
    {
        return Ok(account);
    }
    let mut response = answer::redirect(&state.config.url("/eve"));
    viewer::put(state, viewer, "intended", json!(page), &mut response)
        .await
        .map_err(errors::server_error)?;
    Err(response)
}

/// A form behind the login: the account, or 401 with the login.
pub async fn form_account(state: &AppState, viewer: &Viewer) -> Result<Account, Response> {
    Account::load(state, viewer)
        .await
        .map_err(errors::server_error)?
        .ok_or_else(answer::unauthenticated)
}

/// The admin gate.
pub fn require_admin(state: &AppState, account: &Account) -> Result<(), Response> {
    if account.is_admin(state) {
        Ok(())
    } else {
        Err(errors::forbidden())
    }
}

/// `{character}` in a path: an id that names no character is not found,
/// one of another account is forbidden.
pub async fn owned_character(
    state: &AppState,
    account: &Account,
    id: &str,
) -> Result<Character, Response> {
    let not_found = || errors::model_not_found("Character", id);
    let parsed: i64 = id.parse().map_err(|_| not_found())?;
    let character = character(state, parsed)
        .await
        .map_err(errors::server_error)?
        .ok_or_else(not_found)?;
    if !account.owns(character.id) {
        return Err(errors::forbidden());
    }
    Ok(character)
}

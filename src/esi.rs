//! EVE's API as `nicolaskion/eve` 0.11 called it: every request carries
//! the user agent and `X-Compatibility-Date: 2026-07-17`; reads (contacts,
//! affiliations, names, corporations, alliances) are tried five times, five
//! seconds apart, while ESI answers with a server error; contact writes and
//! mail are tried once. Authenticated calls use the character's token for
//! the scope (`auth::tokens`), and a 401 or 403 deletes that token. A
//! failure is an error; callers carry on without the data.

use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::Method;
use serde::Deserialize;
use serde_json::{Value, json};
use unicode_normalization::UnicodeNormalization;

use crate::auth::tokens::{self, TokenError};
use crate::format::php_float_string;
use crate::server::AppState;

pub const COMPATIBILITY_DATE: &str = "2026-07-17";

/// Tries of a read ESI answers with a server error.
const READ_TRIES: usize = 5;

#[derive(Debug)]
pub enum EsiError {
    /// ESI answered with this status.
    Status(u16),
    Network(String),
    /// The character holds no token with the scope.
    NoToken,
    Token(TokenError),
    Decode(String),
    Db(sqlx::Error),
}

impl fmt::Display for EsiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EsiError::Status(status) => write!(f, "ESI answered {status}"),
            EsiError::Network(error) => write!(f, "ESI unreachable: {error}"),
            EsiError::NoToken => write!(f, "no token with the scope"),
            EsiError::Token(error) => write!(f, "{error}"),
            EsiError::Decode(error) => write!(f, "unexpected ESI answer: {error}"),
            EsiError::Db(error) => write!(f, "database error: {error}"),
        }
    }
}

impl std::error::Error for EsiError {}

impl From<sqlx::Error> for EsiError {
    fn from(error: sqlx::Error) -> Self {
        EsiError::Db(error)
    }
}

impl From<TokenError> for EsiError {
    fn from(error: TokenError) -> Self {
        match error {
            TokenError::Db(error) => EsiError::Db(error),
            other => EsiError::Token(other),
        }
    }
}

/// A contact of a character, corporation or alliance.
#[derive(Debug, Clone, Deserialize)]
pub struct Contact {
    pub contact_id: i64,
    pub contact_type: String,
    pub standing: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Affiliation {
    pub character_id: i64,
    pub corporation_id: i64,
    #[serde(default)]
    pub alliance_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Name {
    pub id: i64,
    pub name: String,
}

/// `GET /corporations/{id}/`, the columns the app stores.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Corporation {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub ticker: Option<String>,
    #[serde(default)]
    pub member_count: Option<i64>,
    #[serde(default)]
    pub tax_rate: Option<f64>,
    #[serde(default)]
    pub war_eligible: Option<bool>,
    #[serde(default)]
    pub date_founded: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub shares: Option<i64>,
    #[serde(default)]
    pub alliance_id: Option<i64>,
}

/// `GET /alliances/{id}/`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Alliance {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub ticker: Option<String>,
    #[serde(default)]
    pub date_founded: Option<String>,
}

/// The decoding of `nicolaskion/eve`'s `Unicode::normalize`: literal
/// `\uXXXX` escapes (also inside Python's `u'...'`) become characters, and
/// the text is NFC.
pub fn normalize(text: &str) -> String {
    let text = if text.contains("\\u") {
        decode_escapes(text)
    } else {
        text.to_owned()
    };
    text.nfc().collect()
}

fn decode_escapes(text: &str) -> String {
    let unwrapped = regex::Regex::new(r"u'((?:\\u[0-9a-fA-F]{4})+)'")
        .expect("a pattern")
        .replace_all(text, "$1")
        .into_owned();
    regex::Regex::new(r"(?:\\u[0-9a-fA-F]{4})+")
        .expect("a pattern")
        .replace_all(&unwrapped, |found: &regex::Captures| {
            serde_json::from_str::<String>(&format!("\"{}\"", &found[0]))
                .unwrap_or_else(|_| found[0].to_owned())
        })
        .into_owned()
}

fn normalize_value(value: &mut Value) {
    match value {
        Value::String(text) => *text = normalize(text),
        Value::Array(items) => items.iter_mut().for_each(normalize_value),
        Value::Object(map) => map.values_mut().for_each(normalize_value),
        _ => {}
    }
}

/// A cap on the rate of outgoing requests (`ESI_MAX_RPS`), so a burst of
/// jobs spaces itself out; 0 turns it off.
struct RateLimiter {
    interval: Option<Duration>,
    next_slot: Mutex<Instant>,
}

impl RateLimiter {
    fn new(max_rps: f64) -> Self {
        Self {
            interval: (max_rps > 0.0).then(|| Duration::from_secs_f64(1.0 / max_rps)),
            next_slot: Mutex::new(Instant::now()),
        }
    }

    async fn acquire(&self) {
        let Some(interval) = self.interval else {
            return;
        };
        let slot = {
            let mut next = self.next_slot.lock().expect("rate limiter lock");
            let slot = (*next).max(Instant::now());
            *next = slot + interval;
            slot
        };
        let now = Instant::now();
        if slot > now {
            tokio::time::sleep(slot - now).await;
        }
    }
}

/// One answer: the status, the `X-Pages` header and the body.
struct Answer {
    status: u16,
    pages: u32,
    body: Value,
}

#[derive(Clone)]
pub struct EsiClient {
    base_url: String,
    http: reqwest::Client,
    retry_delay: Duration,
    limiter: Arc<RateLimiter>,
}

impl EsiClient {
    pub fn new(config: &crate::config::EsiConfig) -> Self {
        Self {
            base_url: config.base_url.trim_end_matches('/').to_owned(),
            http: reqwest::Client::builder()
                .user_agent(&config.user_agent)
                .timeout(Duration::from_secs(30))
                .build()
                .expect("an HTTP client"),
            retry_delay: config.retry_delay,
            limiter: Arc::new(RateLimiter::new(config.max_rps)),
        }
    }

    /// Sends a request, tried again on a server error when `retry` says
    /// so; the last answer counts.
    async fn send(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&Value>,
        token: Option<&str>,
        retry: bool,
    ) -> Result<Answer, EsiError> {
        let tries = if retry { READ_TRIES } else { 1 };
        let mut attempt = 0;
        loop {
            attempt += 1;
            self.limiter.acquire().await;
            let mut request = self
                .http
                .request(method.clone(), format!("{}{path}", self.base_url))
                .header("X-Compatibility-Date", COMPATIBILITY_DATE);
            if !query.is_empty() {
                request = request.query(query);
            }
            if let Some(token) = token {
                request = request.bearer_auth(token);
            }
            if let Some(body) = body {
                request = request.json(body);
            }
            let response = request
                .send()
                .await
                .map_err(|error| EsiError::Network(error.to_string()))?;
            let status = response.status().as_u16();
            if status >= 500 && attempt < tries {
                tokio::time::sleep(self.retry_delay).await;
                continue;
            }
            let pages = response
                .headers()
                .get("x-pages")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse().ok())
                .unwrap_or(1);
            let bytes = response
                .bytes()
                .await
                .map_err(|error| EsiError::Network(error.to_string()))?;
            let mut body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
            normalize_value(&mut body);
            return Ok(Answer {
                status,
                pages,
                body,
            });
        }
    }

    async fn public<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<T, EsiError> {
        let answer = self.send(method, path, &[], body, None, true).await?;
        if !(200..300).contains(&answer.status) {
            tracing::warn!(path, status = answer.status, "ESI answered an error");
            return Err(EsiError::Status(answer.status));
        }
        serde_json::from_value(answer.body).map_err(|error| EsiError::Decode(error.to_string()))
    }

    /// `POST /characters/affiliation/` for up to 1000 characters.
    pub async fn affiliations(&self, ids: &[i64]) -> Result<Vec<Affiliation>, EsiError> {
        self.public(Method::POST, "/characters/affiliation/", Some(&json!(ids)))
            .await
    }

    /// `POST /universe/names/` for up to 1000 ids.
    pub async fn names(&self, ids: &[i64]) -> Result<Vec<Name>, EsiError> {
        self.public(Method::POST, "/universe/names/", Some(&json!(ids)))
            .await
    }

    pub async fn corporation(&self, id: i64) -> Result<Corporation, EsiError> {
        self.public(Method::GET, &format!("/corporations/{id}/"), None)
            .await
    }

    pub async fn alliance(&self, id: i64) -> Result<Alliance, EsiError> {
        self.public(Method::GET, &format!("/alliances/{id}/"), None)
            .await
    }
}

/// What an authenticated call sends.
struct Request<'a> {
    method: Method,
    path: &'a str,
    query: &'a [(&'a str, String)],
    body: Option<&'a Value>,
    /// Tried again on a server error (the reads).
    retry: bool,
}

/// An authenticated request with the character's token for the scope;
/// a 401 or 403 deletes the token used.
async fn authenticated(
    state: &AppState,
    character_id: i64,
    scope: &str,
    request: Request<'_>,
) -> Result<Answer, EsiError> {
    let token = tokens::access_token(
        &state.pool,
        &state.sso,
        character_id,
        scope,
        state.clock.now(),
    )
    .await?
    .ok_or(EsiError::NoToken)?;
    let answer = state
        .esi
        .send(
            request.method,
            request.path,
            request.query,
            request.body,
            Some(&token.access_token),
            request.retry,
        )
        .await?;
    if answer.status == 401 || answer.status == 403 {
        tokens::delete(&state.pool, token.token_id).await?;
    }
    if !(200..300).contains(&answer.status) {
        tracing::warn!(
            path = request.path,
            status = answer.status,
            "ESI answered an error"
        );
        return Err(EsiError::Status(answer.status));
    }
    Ok(answer)
}

/// Every page of a contact list (`characters/{id}`, `corporations/{id}` or
/// `alliances/{id}`), read with the character's token for the scope.
pub async fn contacts(
    state: &AppState,
    reader: i64,
    scope: &str,
    owner: &str,
) -> Result<Vec<Contact>, EsiError> {
    let path = format!("/{owner}/contacts/");
    let mut contacts = Vec::new();
    let mut page = 1;
    loop {
        let query = [("page", page.to_string())];
        let answer = authenticated(
            state,
            reader,
            scope,
            Request {
                method: Method::GET,
                path: &path,
                query: &query,
                body: None,
                retry: true,
            },
        )
        .await?;
        let mut list: Vec<Contact> = serde_json::from_value(answer.body)
            .map_err(|error| EsiError::Decode(error.to_string()))?;
        contacts.append(&mut list);
        if page >= answer.pages {
            return Ok(contacts);
        }
        page += 1;
    }
}

/// Adds (POST) or edits (PUT) up to 100 contacts of a character with one
/// standing; `watched` is off.
pub async fn write_contacts(
    state: &AppState,
    character_id: i64,
    method: Method,
    ids: &[i64],
    standing: f64,
) -> Result<(), EsiError> {
    let path = format!("/characters/{character_id}/contacts/");
    let query = [
        ("standing", php_float_string(standing)),
        ("watched", "0".to_owned()),
    ];
    let body = json!(ids);
    authenticated(
        state,
        character_id,
        crate::scopes::WRITE_CHARACTER_CONTACTS,
        Request {
            method,
            path: &path,
            query: &query,
            body: Some(&body),
            retry: false,
        },
    )
    .await
    .map(|_| ())
}

/// Deletes up to 20 contacts of a character.
pub async fn delete_contacts(
    state: &AppState,
    character_id: i64,
    ids: &[i64],
) -> Result<(), EsiError> {
    let ids: Vec<String> = ids.iter().map(i64::to_string).collect();
    let path = format!("/characters/{character_id}/contacts/");
    let query = [("contact_ids", ids.join(","))];
    authenticated(
        state,
        character_id,
        crate::scopes::WRITE_CHARACTER_CONTACTS,
        Request {
            method: Method::DELETE,
            path: &path,
            query: &query,
            body: None,
            retry: false,
        },
    )
    .await
    .map(|_| ())
}

/// Sends an EVE mail from a character to a character, free of cost.
pub async fn send_mail(
    state: &AppState,
    sender_id: i64,
    recipient_id: i64,
    subject: &str,
    body: &str,
) -> Result<(), EsiError> {
    let path = format!("/characters/{sender_id}/mail/");
    let payload = json!({
        "approved_cost": 0,
        "body": body,
        "recipients": [{ "recipient_id": recipient_id, "recipient_type": "character" }],
        "subject": subject,
    });
    authenticated(
        state,
        sender_id,
        crate::scopes::SEND_MAIL,
        Request {
            method: Method::POST,
            path: &path,
            query: &[],
            body: Some(&payload),
            retry: false,
        },
    )
    .await
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_names_like_the_esi_client() {
        assert_eq!(normalize("plain"), "plain");
        assert_eq!(normalize("u'\\u038f' Corp"), "\u{038f} Corp");
        assert_eq!(normalize("\\u0410\\u043b"), "\u{0410}\u{043b}");
        assert_eq!(normalize("\\uD83D\\uDE80"), "\u{1f680}");
        assert_eq!(normalize("e\u{0301}"), "\u{e9}");
    }
}

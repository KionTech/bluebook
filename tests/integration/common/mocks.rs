//! Local stand-ins for the services the app calls: ESI, EVE SSO (token
//! endpoint and JWKS) and a Discord webhook. Each listens on an ephemeral
//! port, records every request and can be told to fail. They answer like
//! the real services, with data the test sets.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde_json::{Value, json};

/// One request a mock received.
#[derive(Debug, Clone)]
pub struct Recorded {
    pub method: Method,
    pub path: String,
    pub query: Option<String>,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl Recorded {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|value| value.to_str().ok())
    }

    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).expect("a JSON body")
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// One query parameter, decoded.
    pub fn query_param(&self, name: &str) -> Option<String> {
        let query = self.query.as_deref()?;
        form_urlencoded::parse(query.as_bytes())
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    }

    /// `METHOD /path?query`.
    pub fn line(&self) -> String {
        match &self.query {
            Some(query) => format!("{} {}?{query}", self.method, self.path),
            None => format!("{} {}", self.method, self.path),
        }
    }
}

fn record(method: Method, uri: &Uri, headers: HeaderMap, body: &Bytes) -> Recorded {
    Recorded {
        method,
        path: uri.path().to_owned(),
        query: uri.query().map(str::to_owned),
        headers,
        body: body.to_vec(),
    }
}

async fn listen(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind a mock");
    let address = listener.local_addr().expect("a mock address");
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve a mock");
    });
    format!("http://{address}")
}

fn status(code: u16) -> StatusCode {
    StatusCode::from_u16(code).expect("a status")
}

// ---------------------------------------------------------------------------
// ESI
// ---------------------------------------------------------------------------

/// A canned answer.
#[derive(Debug, Clone)]
pub struct Canned {
    pub status: u16,
    pub body: Value,
}

/// A fixed answer: status, body and headers.
type ExactAnswer = (u16, Value, Vec<(String, String)>);

#[derive(Default)]
struct EsiData {
    /// character id to (corporation id, alliance id).
    affiliations: HashMap<i64, (i64, Option<i64>)>,
    /// id to (category, name).
    names: HashMap<i64, (String, String)>,
    alliances: HashMap<i64, Value>,
    corporations: HashMap<i64, Value>,
    /// `characters/<id>`, `corporations/<id>` or `alliances/<id>` to the
    /// contact list.
    contacts: HashMap<String, Vec<Value>>,
    /// Contacts per page of the contact lists.
    page_size: Option<usize>,
    /// `METHOD /path` to answers given once each, in order, before the
    /// regular answer.
    queued: HashMap<String, VecDeque<Canned>>,
    /// `METHOD /path` (or `/path` for every method) to a status every
    /// request gets.
    failures: HashMap<String, u16>,
    /// Every request answers this status.
    fail_all: Option<u16>,
    /// `METHOD /path` to a fixed answer (the golden scenarios' fakes).
    exact: HashMap<String, ExactAnswer>,
    /// Only the exact answers; anything else is a 404, as the Laravel
    /// reference's fakes answered.
    strict: bool,
    /// In strict mode, contact writes still succeed.
    writes_succeed: bool,
    next_mail_id: i64,
    requests: Vec<Recorded>,
}

/// ESI as the app calls it: contacts (read with pages, add, edit,
/// delete), affiliations, names, corporations, alliances and mail.
#[derive(Clone)]
pub struct MockEsi {
    url: String,
    data: Arc<Mutex<EsiData>>,
}

impl MockEsi {
    pub async fn start() -> Self {
        let data = Arc::new(Mutex::new(EsiData {
            next_mail_id: 987_654,
            ..EsiData::default()
        }));
        let router = Router::new().fallback(esi_any).with_state(data.clone());
        Self {
            url: listen(router).await,
            data,
        }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    fn with<T>(&self, change: impl FnOnce(&mut EsiData) -> T) -> T {
        change(&mut self.data.lock().expect("ESI lock"))
    }

    /// A character's corporation and alliance, as the affiliation endpoint
    /// answers them.
    pub fn affiliation(&self, character_id: i64, corporation_id: i64, alliance_id: Option<i64>) {
        self.with(|data| {
            data.affiliations
                .insert(character_id, (corporation_id, alliance_id))
        });
    }

    pub fn name(&self, id: i64, category: &str, name: &str) {
        self.with(|data| {
            data.names
                .insert(id, (category.to_owned(), name.to_owned()))
        });
    }

    /// An alliance as `GET /alliances/{id}/` answers it.
    pub fn alliance(&self, id: i64, name: &str) {
        self.alliance_json(id, esi_alliance_payload(name));
    }

    pub fn alliance_json(&self, id: i64, payload: Value) {
        self.with(|data| data.alliances.insert(id, payload));
    }

    /// A corporation as `GET /corporations/{id}/` answers it.
    pub fn corporation(&self, id: i64, name: &str, alliance_id: Option<i64>) {
        self.corporation_json(id, esi_corporation_payload(name, alliance_id));
    }

    pub fn corporation_json(&self, id: i64, payload: Value) {
        self.with(|data| data.corporations.insert(id, payload));
    }

    /// The contacts of `characters/<id>`, `corporations/<id>` or
    /// `alliances/<id>`: (contact id, type, standing).
    pub fn contacts(&self, owner: &str, contacts: &[(i64, &str, f64)]) {
        let list = contacts
            .iter()
            .map(|(id, kind, standing)| {
                json!({
                    "contact_id": id,
                    "contact_type": kind,
                    "standing": standing,
                    "label_ids": [],
                })
            })
            .collect();
        self.with(|data| data.contacts.insert(owner.to_owned(), list));
    }

    /// How many contacts a page of a contact list holds.
    pub fn page_size(&self, size: usize) {
        self.with(|data| data.page_size = Some(size));
    }

    /// The next requests to `METHOD /path` get these answers, one each.
    pub fn queue(&self, line: &str, answers: Vec<Canned>) {
        self.with(|data| {
            data.queued
                .entry(line.to_owned())
                .or_default()
                .extend(answers)
        });
    }

    /// `METHOD /path` always answers this (the recorded fakes).
    pub fn exact(&self, line: &str, code: u16, body: Value, headers: Vec<(String, String)>) {
        self.with(|data| data.exact.insert(line.to_owned(), (code, body, headers)));
    }

    /// Answers only what `exact` set, 404 for anything else.
    pub fn strict(&self) {
        self.with(|data| data.strict = true);
    }

    /// Contact writes succeed even in strict mode.
    pub fn writes_succeed(&self) {
        self.with(|data| data.writes_succeed = true);
    }

    /// `METHOD /path` (or `/path` for any method) answers a status.
    pub fn fail_path(&self, path: &str, code: u16) {
        self.with(|data| data.failures.insert(path.to_owned(), code));
    }

    /// Every request answers a status.
    pub fn fail_with(&self, code: u16) {
        self.with(|data| data.fail_all = Some(code));
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.with(|data| data.requests.clone())
    }

    /// The requests as `METHOD /path?query`, in order.
    pub fn lines(&self) -> Vec<String> {
        self.requests().iter().map(Recorded::line).collect()
    }

    pub fn clear_requests(&self) {
        self.with(|data| data.requests.clear());
    }
}

/// A minimal public corporation, like the PHP tests' `esiCorporationPayload`.
pub fn esi_corporation_payload(name: &str, alliance_id: Option<i64>) -> Value {
    json!({
        "name": name,
        "ticker": "CORP",
        "ceo_id": 1,
        "creator_id": 1,
        "date_founded": "2010-01-01T00:00:00Z",
        "description": "",
        "member_count": 10,
        "tax_rate": 0.1,
        "url": "",
        "alliance_id": alliance_id,
    })
}

/// A minimal public alliance, like the PHP tests' `esiAlliancePayload`.
pub fn esi_alliance_payload(name: &str) -> Value {
    json!({
        "name": name,
        "ticker": "ALLY",
        "creator_corporation_id": 1,
        "creator_id": 1,
        "date_founded": "2010-01-01T00:00:00Z",
    })
}

fn esi_json(code: u16, body: Value) -> Response {
    (status(code), axum::Json(body)).into_response()
}

async fn esi_any(
    State(data): State<Arc<Mutex<EsiData>>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let mut data = data.lock().expect("ESI lock");
    let recorded = record(method.clone(), &uri, headers, &body);
    let page: usize = recorded
        .query_param("page")
        .and_then(|page| page.parse().ok())
        .unwrap_or(1);
    data.requests.push(recorded);
    let path = uri.path().to_owned();
    let line = format!("{method} {path}");

    if let Some(code) = data.fail_all {
        return esi_json(code, json!({ "error": "mock failure" }));
    }
    if let Some(canned) = data.queued.get_mut(&line).and_then(VecDeque::pop_front) {
        return esi_json(canned.status, canned.body);
    }
    if let Some(code) = data
        .failures
        .get(&line)
        .or_else(|| data.failures.get(&path))
    {
        return esi_json(*code, json!({ "error": "mock failure" }));
    }
    if let Some((code, body, headers)) = data.exact.get(&line).cloned() {
        let mut response = if body.is_null() {
            status(code).into_response()
        } else {
            esi_json(code, body)
        };
        for (name, value) in headers {
            response.headers_mut().insert(
                axum::http::HeaderName::from_bytes(name.as_bytes()).expect("a header name"),
                HeaderValue::from_str(&value).expect("a header"),
            );
        }
        return response;
    }
    let is_write = matches!(method.as_str(), "POST" | "PUT" | "DELETE")
        && path.starts_with("/characters/")
        && path.ends_with("/contacts/");
    if data.strict && !(data.writes_succeed && is_write) {
        return status(404).into_response();
    }

    let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
    match (method.as_str(), segments.as_slice()) {
        ("POST", ["characters", "affiliation"]) => {
            let ids: Vec<i64> = serde_json::from_slice(&body).unwrap_or_default();
            let answer: Vec<Value> = ids
                .iter()
                .filter_map(|id| {
                    let (corporation_id, alliance_id) = data.affiliations.get(id)?;
                    let mut row = json!({ "character_id": id, "corporation_id": corporation_id });
                    if let Some(alliance_id) = alliance_id {
                        row["alliance_id"] = json!(alliance_id);
                    }
                    Some(row)
                })
                .collect();
            esi_json(200, json!(answer))
        }
        ("POST", ["universe", "names"]) => {
            let ids: Vec<i64> = serde_json::from_slice(&body).unwrap_or_default();
            let answer: Vec<Value> = ids
                .iter()
                .filter_map(|id| {
                    let (category, name) = data.names.get(id)?;
                    Some(json!({ "category": category, "id": id, "name": name }))
                })
                .collect();
            esi_json(200, json!(answer))
        }
        (
            "GET",
            [
                kind @ ("characters" | "corporations" | "alliances"),
                id,
                "contacts",
            ],
        ) => {
            let list = data
                .contacts
                .get(&format!("{kind}/{id}"))
                .cloned()
                .unwrap_or_default();
            let size = data.page_size.unwrap_or(usize::MAX).max(1);
            let pages = list.len().div_ceil(size).max(1);
            let start = (page - 1).saturating_mul(size).min(list.len());
            let end = start.saturating_add(size).min(list.len());
            let mut response = esi_json(200, json!(list[start..end]));
            response.headers_mut().insert(
                "x-pages",
                HeaderValue::from_str(&pages.to_string()).expect("a header"),
            );
            response
        }
        ("POST", ["characters", _, "contacts"]) => {
            let ids: Vec<i64> = serde_json::from_slice(&body).unwrap_or_default();
            esi_json(201, json!(ids))
        }
        ("PUT" | "DELETE", ["characters", _, "contacts"]) => StatusCode::NO_CONTENT.into_response(),
        ("POST", ["characters", _, "mail"]) => {
            data.next_mail_id += 1;
            esi_json(201, json!(data.next_mail_id - 1))
        }
        ("GET", ["corporations", id]) => match id
            .parse::<i64>()
            .ok()
            .and_then(|id| data.corporations.get(&id))
        {
            Some(corporation) => esi_json(200, corporation.clone()),
            None => esi_json(404, json!({ "error": "Corporation not found" })),
        },
        ("GET", ["alliances", id]) => match id
            .parse::<i64>()
            .ok()
            .and_then(|id| data.alliances.get(&id))
        {
            Some(alliance) => esi_json(200, alliance.clone()),
            None => esi_json(404, json!({ "error": "Alliance not found" })),
        },
        _ => esi_json(404, json!({ "error": "Not found" })),
    }
}

// ---------------------------------------------------------------------------
// EVE SSO
// ---------------------------------------------------------------------------

/// The key id of the mock's signing key.
pub const SSO_KEY_ID: &str = "JWT-Signature-Key";

/// The character the mock SSO signs tokens for.
#[derive(Debug, Clone)]
pub struct SsoCharacter {
    pub id: i64,
    pub name: String,
    pub owner: String,
    pub scopes: Vec<String>,
}

struct SsoData {
    character: SsoCharacter,
    /// The authorization code the token endpoint accepts.
    code: String,
    /// Answers of the refresh grant, given once each, before the regular
    /// one.
    refresh_answers: VecDeque<(u16, Value)>,
    token_status: Option<u16>,
    /// The answer of every refresh grant, instead of a numbered one.
    refresh_answer: Option<Value>,
    refreshes: i64,
    requests: Vec<Recorded>,
}

/// EVE SSO: the token endpoint (authorization code and refresh grants,
/// basic auth) and the JWKS the access token is checked against. Tokens
/// are RS256 JWTs like EVE's, signed with a fixed test key
/// (tests/fixtures/sso/private-key.pem).
#[derive(Clone)]
pub struct MockSso {
    url: String,
    data: Arc<Mutex<SsoData>>,
}

impl MockSso {
    pub async fn start() -> Self {
        let data = Arc::new(Mutex::new(SsoData {
            character: SsoCharacter {
                id: 95_000_001,
                name: "Test Pilot".to_owned(),
                owner: "owner-hash".to_owned(),
                scopes: Vec::new(),
            },
            code: "valid-code".to_owned(),
            refresh_answers: VecDeque::new(),
            token_status: None,
            refresh_answer: None,
            refreshes: 0,
            requests: Vec::new(),
        }));
        let router = Router::new()
            .route("/v2/oauth/token", post(sso_token))
            .route("/oauth/jwks", get(sso_jwks))
            .with_state(data.clone());
        Self {
            url: listen(router).await,
            data,
        }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    /// The character the next tokens are for, with their scopes.
    pub fn sign_in_as(&self, id: i64, name: &str, owner: &str, scopes: &[&str]) {
        let mut data = self.data.lock().expect("SSO lock");
        data.character = SsoCharacter {
            id,
            name: name.to_owned(),
            owner: owner.to_owned(),
            scopes: scopes.iter().map(|scope| (*scope).to_owned()).collect(),
        };
    }

    /// The code the token endpoint accepts.
    pub fn code(&self) -> String {
        self.data.lock().expect("SSO lock").code.clone()
    }

    /// Every grant answers a status.
    pub fn fail_token_with(&self, code: u16) {
        self.data.lock().expect("SSO lock").token_status = Some(code);
    }

    /// The next refresh grants answer these statuses, one each; a 400 says
    /// `invalid_grant` (the refresh token is dead), any other error status
    /// a server error.
    pub fn queue_refresh(&self, statuses: &[u16]) {
        for status in statuses {
            let error = if *status == 400 {
                "invalid_grant"
            } else {
                "server_error"
            };
            self.queue_refresh_with(*status, json!({ "error": error }));
        }
    }

    /// The next refresh grant answers this status and body.
    pub fn queue_refresh_with(&self, status: u16, body: Value) {
        self.data
            .lock()
            .expect("SSO lock")
            .refresh_answers
            .push_back((status, body));
    }

    /// Every refresh grant answers this.
    pub fn refresh_answer(&self, answer: Value) {
        self.data.lock().expect("SSO lock").refresh_answer = Some(answer);
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.data.lock().expect("SSO lock").requests.clone()
    }
}

/// A token signed with the test key for a character.
pub fn signed_access_token(character: &SsoCharacter) -> String {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let claims = json!({
        "scp": character.scopes,
        "jti": "test-jti",
        "kid": SSO_KEY_ID,
        "sub": format!("CHARACTER:EVE:{}", character.id),
        "azp": super::EVE_CLIENT_ID,
        "tenant": "tranquility",
        "tier": "live",
        "region": "world",
        "aud": [super::EVE_CLIENT_ID, "EVE Online"],
        "name": character.name,
        "owner": character.owner,
        "exp": now + 1199,
        "iat": now,
        "iss": "https://login.eveonline.com",
    });
    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some(SSO_KEY_ID.to_owned());
    let pem = std::fs::read(super::repo_path("tests/fixtures/sso/private-key.pem"))
        .expect("the test key");
    jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_rsa_pem(&pem).expect("an RSA key"),
    )
    .expect("a signed token")
}

async fn sso_token(
    State(data): State<Arc<Mutex<SsoData>>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let mut data = data.lock().expect("SSO lock");
    data.requests.push(record(method, &uri, headers, &body));
    if let Some(code) = data.token_status {
        return (
            status(code),
            axum::Json(json!({ "error": "invalid_request" })),
        )
            .into_response();
    }
    let form: HashMap<String, String> = form_urlencoded::parse(&body).into_owned().collect();
    match form.get("grant_type").map(String::as_str) {
        Some("refresh_token") => {
            if let Some((code, body)) = data.refresh_answers.pop_front()
                && code != 200
            {
                return (status(code), axum::Json(body)).into_response();
            }
            data.refreshes += 1;
            if let Some(answer) = &data.refresh_answer {
                return axum::Json(answer.clone()).into_response();
            }
            axum::Json(json!({
                "access_token": format!("refreshed-access-{}", data.refreshes),
                "expires_in": 1199,
                "token_type": "Bearer",
                "refresh_token": format!("refreshed-refresh-{}", data.refreshes),
            }))
            .into_response()
        }
        _ => {
            if form.get("code") != Some(&data.code) {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({ "error": "invalid_grant" })),
                )
                    .into_response();
            }
            axum::Json(json!({
                "access_token": signed_access_token(&data.character),
                "expires_in": 1199,
                "token_type": "Bearer",
                "refresh_token": "test-refresh-token",
            }))
            .into_response()
        }
    }
}

async fn sso_jwks(
    State(data): State<Arc<Mutex<SsoData>>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let mut data = data.lock().expect("SSO lock");
    data.requests
        .push(record(method, &uri, headers, &Bytes::new()));
    let modulus = std::fs::read_to_string(super::repo_path("tests/fixtures/sso/modulus.txt"))
        .expect("the test key's modulus");
    axum::Json(json!({
        "keys": [
            {
                "alg": "RS256",
                "e": "AQAB",
                "kid": SSO_KEY_ID,
                "kty": "RSA",
                "n": modulus.trim(),
                "use": "sig",
            },
        ],
        "SkipUnresolvedJsonWebKeys": true,
    }))
    .into_response()
}

// ---------------------------------------------------------------------------
// Discord
// ---------------------------------------------------------------------------

#[derive(Default)]
struct DiscordData {
    status: Option<u16>,
    requests: Vec<Recorded>,
}

/// A Discord webhook: records what is posted to `/webhook`.
#[derive(Clone)]
pub struct MockDiscord {
    url: String,
    data: Arc<Mutex<DiscordData>>,
}

impl MockDiscord {
    pub async fn start() -> Self {
        let data = Arc::new(Mutex::new(DiscordData::default()));
        let router = Router::new().fallback(discord_any).with_state(data.clone());
        Self {
            url: listen(router).await,
            data,
        }
    }

    /// The webhook's URL.
    pub fn webhook(&self) -> String {
        format!("{}/webhook", self.url)
    }

    pub fn fail_with(&self, code: u16) {
        self.data.lock().expect("Discord lock").status = Some(code);
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.data.lock().expect("Discord lock").requests.clone()
    }
}

async fn discord_any(
    State(data): State<Arc<Mutex<DiscordData>>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let mut data = data.lock().expect("Discord lock");
    data.requests.push(record(method, &uri, headers, &body));
    match data.status {
        Some(code) => status(code).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

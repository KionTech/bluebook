//! Shared harness for the integration suites: the real router on the test
//! database with a fixed clock, mock ESI, EVE SSO and Discord servers on
//! ephemeral ports, request helpers that speak like the SvelteKit pages
//! (props loads and JSON forms), the factories the PHP tests used and the
//! JSON assertions.

// Not every suite uses every helper.
#![allow(dead_code)]

pub mod factories;
pub mod golden;
pub mod mocks;

use std::path::{Path, PathBuf};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Map, Value, json};
use sqlx::PgPool;
use time::OffsetDateTime;
use time::macros::datetime;
use tower::ServiceExt;

use bluebook::clock::Clock;
use bluebook::config::{Config, DemoConfig, EsiConfig, EveConfig};
use bluebook::server::AppState;

#[allow(unused_imports)]
pub use mocks::{Canned, MockDiscord, MockEsi, MockSso};

/// The site URL, as production has it and the reference recorded it.
pub const APP_URL: &str = "https://bluebook.nicolaskion.dev";
pub const HOST: &str = "bluebook.nicolaskion.dev";
pub const EVE_CLIENT_ID: &str = "test-client-id";
pub const EVE_CLIENT_SECRET: &str = "test-client-secret";

/// The moment the ported PHP tests run at: they pinned 2026-06-25 14:00
/// UTC, outside EVE's downtime.
pub const DEFAULT_NOW: OffsetDateTime = datetime!(2026-06-25 14:00:00 UTC);
/// `DEFAULT_NOW` as `toIso8601String()` writes it.
pub const NOW_ISO: &str = "2026-06-25T14:00:00+00:00";

pub const READ_CHARACTER_CONTACTS: &str = "esi-characters.read_contacts.v1";
pub const WRITE_CHARACTER_CONTACTS: &str = "esi-characters.write_contacts.v1";
pub const READ_CORPORATION_CONTACTS: &str = "esi-corporations.read_contacts.v1";
pub const READ_ALLIANCE_CONTACTS: &str = "esi-alliances.read_contacts.v1";
pub const SEND_MAIL: &str = "esi-mail.send_mail.v1";
pub const PUBLIC_DATA: &str = "publicData";

pub fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// One app under test: its router, the database it writes to, the clock
/// it reads and the mock services it calls.
pub struct TestApp {
    pub router: Router,
    pub state: AppState,
    pub pool: PgPool,
    pub esi: MockEsi,
    pub sso: MockSso,
    pub discord: MockDiscord,
    pub config: Config,
    pub clock: Clock,
}

pub async fn test_pool() -> PgPool {
    bluebook::db::test_pool()
        .await
        .expect("Postgres not reachable - start it with `docker compose up -d postgres`")
}

/// Clears every table; ids restart at 1, like Laravel's RefreshDatabase.
pub async fn reset_database(pool: &PgPool) {
    sqlx::query(
        "truncate alliances, corporations, factions, users, characters, esi_tokens,
            source_contacts, character_synced_contacts, standings_sources, standing_requests,
            discord_settings, sessions, throttles, jobs
         restart identity cascade",
    )
    .execute(pool)
    .await
    .expect("tables truncate");
}

pub fn test_config(esi: &MockEsi, sso: &MockSso) -> Config {
    Config {
        app_name: "Bluebook".to_owned(),
        app_url: APP_URL.to_owned(),
        eve: EveConfig {
            sso_base_url: sso.url(),
            client_id: EVE_CLIENT_ID.to_owned(),
            client_secret: EVE_CLIENT_SECRET.to_owned(),
            callback_url: format!("{APP_URL}/eve/callback"),
            refresh_retry_delay: Duration::ZERO,
        },
        esi: EsiConfig {
            base_url: esi.url(),
            user_agent: "Bluebook (+https://bluebook.nicolaskion.dev)".to_owned(),
            max_rps: 0.0,
            retry_delay: Duration::ZERO,
        },
        admin_character_ids: Vec::new(),
        discord_webhook: None,
        scheduler_enabled: false,
        queue_workers: 1,
        session_lifetime_days: 400,
        demo: DemoConfig {
            enabled: false,
            character_id: 2_100_000_001,
        },
    }
}

/// A signed-in browser: the session cookie for the next requests.
#[derive(Debug, Clone)]
pub struct Browser {
    pub cookie: Option<String>,
}

impl Browser {
    pub fn guest() -> Self {
        Self { cookie: None }
    }
}

impl TestApp {
    /// A fresh app on an empty database, its clock at `DEFAULT_NOW`.
    pub async fn new() -> Self {
        Self::at(DEFAULT_NOW).await
    }

    pub async fn at(now: OffsetDateTime) -> Self {
        let pool = test_pool().await;
        reset_database(&pool).await;
        let esi = MockEsi::start().await;
        let sso = MockSso::start().await;
        let discord = MockDiscord::start().await;
        let config = test_config(&esi, &sso);
        let clock = Clock::fixed(now);
        let state = AppState::new(pool.clone(), config.clone(), clock.clone());
        Self {
            router: bluebook::server::router(state.clone()),
            state,
            pool,
            esi,
            sso,
            discord,
            config,
            clock,
        }
    }

    /// Rebuilds the router with a changed configuration, on the same
    /// database, clock and mocks (Laravel's `config([...])`).
    pub fn reconfigure(&mut self, change: impl FnOnce(&mut Config)) {
        change(&mut self.config);
        self.state = AppState::new(self.pool.clone(), self.config.clone(), self.clock.clone());
        self.router = bluebook::server::router(self.state.clone());
    }

    /// `config(['services.eveonline.admin_character_ids' => $ids])`.
    pub fn admins(&mut self, ids: &[i64]) {
        let ids = ids.to_vec();
        self.reconfigure(|config| config.admin_character_ids = ids);
    }

    /// Signs a user in (Laravel's `actingAs`), optionally with an active
    /// character in the session.
    pub async fn sign_in(&self, user_id: i64) -> Browser {
        self.sign_in_with(user_id, Map::new()).await
    }

    pub async fn sign_in_as(&self, user_id: i64, character_id: i64) -> Browser {
        let mut data = Map::new();
        data.insert("active_character_id".to_owned(), json!(character_id));
        self.sign_in_with(user_id, data).await
    }

    pub async fn sign_in_with(&self, user_id: i64, data: Map<String, Value>) -> Browser {
        let token =
            bluebook::auth::session::start(&self.pool, Some(user_id), data, self.clock.now())
                .await
                .expect("a session");
        Browser {
            cookie: Some(format!(
                "{}={token}",
                bluebook::auth::session::SESSION_COOKIE
            )),
        }
    }

    /// The session row of a browser.
    pub async fn session_data(&self, browser: &Browser) -> Option<Map<String, Value>> {
        let token = browser.cookie.as_ref()?.split_once('=')?.1.to_owned();
        let data: Option<sqlx::types::Json<Map<String, Value>>> =
            sqlx::query_scalar("select data from sessions where id = $1")
                .bind(bluebook::auth::session::hash_token(&token))
                .fetch_optional(&self.pool)
                .await
                .expect("a session lookup");
        data.map(|data| data.0)
    }

    /// A browser GET of a path, as a guest or with the browser's cookie.
    pub async fn get(&self, browser: &Browser, path: &str) -> TestResponse {
        self.get_with(browser, path, &[]).await
    }

    pub async fn get_with(
        &self,
        browser: &Browser,
        path: &str,
        headers: &[(&str, &str)],
    ) -> TestResponse {
        let mut all = vec![("host", HOST)];
        if let Some(cookie) = &browser.cookie {
            all.push(("cookie", cookie));
        }
        all.extend_from_slice(headers);
        self.request(Method::GET, path, header_map(&all), Body::empty())
            .await
    }

    /// A page's props, as the SvelteKit load fetches them: `/dashboard`
    /// loads `/api/pages/dashboard`, `/` loads `/api/pages/home`.
    pub async fn page(&self, browser: &Browser, path: &str) -> TestResponse {
        let (path, query) = path.split_once('?').unwrap_or((path, ""));
        let path = if path == "/" { "/home" } else { path };
        let url = if query.is_empty() {
            format!("/api/pages{path}")
        } else {
            format!("/api/pages{path}?{query}")
        };
        self.get_with(browser, &url, &[("accept", "application/json")])
            .await
    }

    /// A form the pages send: JSON both ways, from the site itself.
    pub async fn form(
        &self,
        browser: &Browser,
        method: Method,
        path: &str,
        body: Value,
    ) -> TestResponse {
        let mut headers = vec![
            ("host", HOST),
            ("accept", "application/json"),
            ("content-type", "application/json"),
            ("origin", APP_URL),
            ("sec-fetch-site", "same-origin"),
        ];
        if let Some(cookie) = &browser.cookie {
            headers.push(("cookie", cookie));
        }
        self.request(
            method,
            path,
            header_map(&headers),
            Body::from(body.to_string()),
        )
        .await
    }

    pub async fn post(&self, browser: &Browser, path: &str, body: Value) -> TestResponse {
        self.form(browser, Method::POST, path, body).await
    }

    pub async fn put(&self, browser: &Browser, path: &str, body: Value) -> TestResponse {
        self.form(browser, Method::PUT, path, body).await
    }

    pub async fn delete(&self, browser: &Browser, path: &str) -> TestResponse {
        self.form(browser, Method::DELETE, path, json!({})).await
    }

    pub async fn request(
        &self,
        method: Method,
        path: &str,
        headers: HeaderMap,
        body: Body,
    ) -> TestResponse {
        let mut request = Request::builder().method(method).uri(path);
        for (name, value) in &headers {
            request = request.header(name, value);
        }
        let response = self
            .router
            .clone()
            .oneshot(request.body(body).expect("request builds"))
            .await
            .expect("router answers");
        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body reads")
            .to_bytes()
            .to_vec();
        TestResponse {
            status,
            headers,
            body,
        }
    }

    pub async fn count(&self, table: &str) -> i64 {
        sqlx::query_scalar(sqlx::AssertSqlSafe(format!("select count(*) from {table}")))
            .fetch_one(&self.pool)
            .await
            .expect("count")
    }

    /// The queued jobs as (kind, payload), oldest first.
    pub async fn jobs(&self) -> Vec<(String, Value)> {
        sqlx::query_as::<_, (String, sqlx::types::Json<Value>)>(
            "select kind, payload from jobs order by id",
        )
        .fetch_all(&self.pool)
        .await
        .expect("jobs")
        .into_iter()
        .map(|(kind, payload)| (kind, payload.0))
        .collect()
    }

    /// The queued jobs of one kind.
    pub async fn jobs_of(&self, kind: &str) -> Vec<Value> {
        self.jobs()
            .await
            .into_iter()
            .filter(|(job_kind, _)| job_kind == kind)
            .map(|(_, payload)| payload)
            .collect()
    }
}

pub fn header_map(headers: &[(&str, &str)]) -> HeaderMap {
    let mut map = HeaderMap::new();
    for (name, value) in headers {
        map.append(
            axum::http::HeaderName::from_bytes(name.as_bytes()).expect("header name"),
            value.parse().expect("header value"),
        );
    }
    map
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl TestResponse {
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or_else(|error| {
            panic!(
                "{} is not JSON ({error}): {}",
                self.status,
                String::from_utf8_lossy(&self.body)
            )
        })
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|value| value.to_str().ok())
    }

    /// Asserts the status, showing the body when it differs.
    #[track_caller]
    pub fn assert_status(&self, status: u16) -> &Self {
        assert_eq!(self.status.as_u16(), status, "{}", self.text());
        self
    }

    /// Asserts a successful form answer and returns where it sends the
    /// browser (`None`: stay and reload, Laravel's `back()`).
    #[track_caller]
    pub fn assert_done(&self) -> Option<String> {
        self.assert_status(200);
        self.json()["redirect"].as_str().map(str::to_owned)
    }

    /// Asserts a failed validation and returns each field's message.
    #[track_caller]
    pub fn assert_invalid(&self) -> Value {
        self.assert_status(422);
        self.json()["errors"].clone()
    }

    /// The target of a redirect.
    #[track_caller]
    pub fn location(&self) -> &str {
        assert!(
            self.status.is_redirection(),
            "{} is no redirect: {}",
            self.status,
            self.text()
        );
        self.header("location").expect("a location")
    }

    /// Every `Set-Cookie` header.
    pub fn set_cookies(&self) -> Vec<String> {
        self.headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|value| value.to_str().ok())
            .map(str::to_owned)
            .collect()
    }

    /// The `name=value` pair of a cookie this answer sets, for the next
    /// request's `Cookie` header.
    pub fn cookie(&self, name: &str) -> Option<String> {
        self.set_cookies().into_iter().find_map(|header| {
            let pair = header.split(';').next()?.trim().to_owned();
            pair.starts_with(&format!("{name}=")).then_some(pair)
        })
    }

    /// The browser this answer leaves behind (its new session cookie, or
    /// the old one).
    pub fn browser(&self, before: &Browser) -> Browser {
        match self.cookie(bluebook::auth::session::SESSION_COOKIE) {
            Some(cookie) if cookie.ends_with('=') => Browser::guest(),
            Some(cookie) => Browser {
                cookie: Some(cookie),
            },
            None => before.clone(),
        }
    }
}

/// Asserts two JSON values are equal with the same keys in the same order
/// at every level, naming the first path that differs.
#[track_caller]
pub fn assert_json_eq(expected: &Value, actual: &Value) {
    if let Some(difference) = json_difference("$", expected, actual) {
        panic!(
            "{difference}\n  expected {}\n  actual   {}",
            serde_json::to_string(expected).unwrap_or_default(),
            serde_json::to_string(actual).unwrap_or_default()
        );
    }
}

pub fn json_difference(path: &str, expected: &Value, actual: &Value) -> Option<String> {
    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => {
            let expected_keys: Vec<_> = expected.keys().collect();
            let actual_keys: Vec<_> = actual.keys().collect();
            if expected_keys != actual_keys {
                return Some(format!(
                    "{path}: keys differ\n  expected {expected_keys:?}\n  actual   {actual_keys:?}"
                ));
            }
            expected.iter().find_map(|(key, value)| {
                json_difference(&format!("{path}.{key}"), value, &actual[key])
            })
        }
        (Value::Array(expected), Value::Array(actual)) => {
            if expected.len() != actual.len() {
                return Some(format!(
                    "{path}: expected {} items, got {}",
                    expected.len(),
                    actual.len()
                ));
            }
            expected
                .iter()
                .zip(actual)
                .enumerate()
                .find_map(|(index, (expected, actual))| {
                    json_difference(&format!("{path}[{index}]"), expected, actual)
                })
        }
        (expected, actual) if expected != actual => {
            Some(format!("{path}: expected {expected}, got {actual}"))
        }
        _ => None,
    }
}

/// The routes of the Laravel app: (method, path).
pub fn read_routes() -> Vec<(String, String)> {
    let text = std::fs::read_to_string(repo_path("tests/fixtures/routes.json")).expect("routes");
    let routes: Vec<Value> = serde_json::from_str(&text).expect("routes JSON");
    let mut all = Vec::new();
    for route in routes {
        let uri = route["uri"].as_str().expect("a uri");
        let uri = if uri == "/" {
            "/".to_owned()
        } else {
            format!("/{uri}")
        };
        for method in route["method"].as_str().expect("methods").split('|') {
            all.push((method.to_owned(), uri.clone()));
        }
    }
    all
}

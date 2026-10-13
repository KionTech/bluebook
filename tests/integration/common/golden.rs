//! Comparisons with what the Laravel reference answered
//! (tests/fixtures/golden, recorded by scripts/laravel-reference/record.py
//! from tests/fixtures/dataset.json).
//!
//! The normalizations, the only differences these comparisons accept
//! besides FIXES.md:
//!
//! - N1: Inertia's `errors` prop is dropped from page props; validation
//!   errors are compared on their own (the forms' 422 answer).
//! - N2: the reference's origin (`http://127.0.0.1:8315`) is the site's,
//!   `APP_URL`, in every string (paginator and grant links, redirects).
//! - N3: the `state` of the EVE login redirect is random; it is compared
//!   by its form (40 letters and digits), not its value.
//! - N4: the reference's HTML error pages are compared by status, title
//!   and message: in `APP_ENV=local` Laravel Boost injects its browser
//!   log script into them.
//! - N5: outgoing calls are compared per service (ESI, EVE SSO, Discord)
//!   by method, path, query, body and the headers the app sets; the
//!   hosts are the mocks', and the user agent is ESI_USER_AGENT (FIXES.md,
//!   "The ESI user agent names the site"). Discord's user agent was
//!   Guzzle's and is not compared.
//! - N6: the clock hour in the key of the hourly "source unreadable"
//!   throttle is the test clock's.
//! - N7: the Discord webhook of the dataset points at the mock.
//! - N8: the scenarios' MySQL statements run as their Postgres
//!   equivalents (`translate_sql`).

use std::path::PathBuf;

use serde_json::{Map, Value, json};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::mocks::Recorded;
use super::{APP_URL, TestApp, repo_path};

/// The reference's origin (N2).
pub const REFERENCE_ORIGIN: &str = "http://127.0.0.1:8315";
pub const REFERENCE_CLIENT_ID: &str = "reference-client-id";
pub const REFERENCE_CLIENT_SECRET: &str = "reference-client-secret";
/// The admin character of the reference (`EVE_ADMIN_CHARACTER_ID`).
pub const ADMIN: i64 = 2_112_000_001;

fn golden_dir() -> PathBuf {
    repo_path("tests/fixtures/golden")
}

/// The moment the fixtures were recorded.
pub fn recorded_at() -> OffsetDateTime {
    let text = std::fs::read_to_string(golden_dir().join("meta.json")).expect("meta.json");
    let meta: Value = serde_json::from_str(&text).expect("meta JSON");
    OffsetDateTime::parse(meta["now"].as_str().expect("now"), &Rfc3339).expect("a time")
}

pub fn load(kind: &str, name: &str) -> Value {
    let path = golden_dir().join(kind).join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut value: Value = serde_json::from_str(&text).expect("golden JSON");
    replace_origin(&mut value);
    value
}

/// The names of the recorded fixtures of a kind.
pub fn names(kind: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(golden_dir().join(kind))
        .expect("a golden directory")
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            name.strip_suffix(".json").map(str::to_owned)
        })
        .collect();
    names.sort();
    names
}

/// N2, in place.
pub fn replace_origin(value: &mut Value) {
    match value {
        Value::String(text) => {
            if text.contains(REFERENCE_ORIGIN) {
                *text = text.replace(REFERENCE_ORIGIN, APP_URL);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(replace_origin),
        Value::Object(map) => map.values_mut().for_each(replace_origin),
        _ => {}
    }
}

/// N1: the page props without Inertia's `errors`.
pub fn page_props(golden: &Value) -> Value {
    let mut props = golden["props"].clone();
    if let Some(map) = props.as_object_mut() {
        map.shift_remove("errors");
    }
    props
}

/// A golden app: the clock at the recording, the reference's admin and
/// EVE application, the dataset loaded and the scenario's SQL run.
pub async fn app(sql: &Value) -> TestApp {
    let mut app = TestApp::at(recorded_at()).await;
    app.reconfigure(|config| {
        config.admin_character_ids = vec![ADMIN];
        config.eve.client_id = REFERENCE_CLIENT_ID.to_owned();
        config.eve.client_secret = REFERENCE_CLIENT_SECRET.to_owned();
    });
    seed(&app).await;
    for statement in sql.as_array().into_iter().flatten() {
        for statement in translate_sql(statement.as_str().expect("SQL"), &app) {
            sqlx::query(sqlx::AssertSqlSafe(statement.clone()))
                .execute(&app.pool)
                .await
                .unwrap_or_else(|error| panic!("{statement}: {error}"));
        }
    }
    app
}

fn dataset() -> Value {
    let text = std::fs::read_to_string(repo_path("tests/fixtures/dataset.json")).expect("dataset");
    serde_json::from_str(&text).expect("dataset JSON")
}

/// A dataset time: `{offset_minutes}` from the clock.
fn time_of(app: &TestApp, value: &Value) -> Option<OffsetDateTime> {
    let minutes = value.get("offset_minutes")?.as_i64()?;
    Some(app.clock.now() + time::Duration::minutes(minutes))
}

/// Loads tests/fixtures/dataset.json the way record.py seeds it.
pub async fn seed(app: &TestApp) {
    let data = dataset();
    let now = app.clock.now();
    let pool = &app.pool;
    for row in data["alliances"].as_array().expect("alliances") {
        sqlx::query(
            "insert into alliances (id, name, ticker, created_at, updated_at) values ($1, $2, $3, $4, $4)",
        )
        .bind(row["id"].as_i64())
        .bind(row["name"].as_str())
        .bind(row["ticker"].as_str())
        .bind(now)
        .execute(pool)
        .await
        .expect("an alliance");
    }
    for row in data["corporations"].as_array().expect("corporations") {
        sqlx::query(
            "insert into corporations (id, name, ticker, alliance_id, created_at, updated_at)
             values ($1, $2, $3, $4, $5, $5)",
        )
        .bind(row["id"].as_i64())
        .bind(row["name"].as_str())
        .bind(row["ticker"].as_str())
        .bind(row["alliance_id"].as_i64())
        .bind(now)
        .execute(pool)
        .await
        .expect("a corporation");
    }
    for row in data["factions"].as_array().expect("factions") {
        sqlx::query(
            "insert into factions (id, name, created_at, updated_at) values ($1, $2, $3, $3)",
        )
        .bind(row["id"].as_i64())
        .bind(row["name"].as_str())
        .bind(now)
        .execute(pool)
        .await
        .expect("a faction");
    }
    for row in data["users"].as_array().expect("users") {
        sqlx::query(
            "insert into users (id, name, setup_completed_at, created_at, updated_at)
             values ($1, $2, $3, $4, $4)",
        )
        .bind(row["id"].as_i64())
        .bind(row["name"].as_str())
        .bind(time_of(app, &row["setup_completed_at"]))
        .bind(now)
        .execute(pool)
        .await
        .expect("a user");
    }
    for (number, row) in data["characters"]
        .as_array()
        .expect("characters")
        .iter()
        .enumerate()
    {
        let owned = !row["user_id"].is_null();
        sqlx::query(
            "insert into characters (id, name, corporation_id, alliance_id, user_id,
                character_owner_hash, should_sync, created_at, updated_at)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $8)",
        )
        .bind(row["id"].as_i64())
        .bind(row["name"].as_str())
        .bind(row["corporation_id"].as_i64())
        .bind(row["alliance_id"].as_i64())
        .bind(row["user_id"].as_i64())
        .bind(owned.then(|| format!("owner-hash-{}", number + 1)))
        .bind(row["should_sync"].as_bool())
        .bind(now)
        .execute(pool)
        .await
        .expect("a character");
    }
    for row in data["users"].as_array().expect("users") {
        sqlx::query("update users set main_character_id = $2 where id = $1")
            .bind(row["id"].as_i64())
            .bind(row["main_character_id"].as_i64())
            .execute(pool)
            .await
            .expect("a main character");
    }
    for (index, row) in data["esi_tokens"]
        .as_array()
        .expect("tokens")
        .iter()
        .enumerate()
    {
        let number = index as i64 + 1;
        let scopes: Vec<String> = row["scopes"]
            .as_array()
            .expect("scopes")
            .iter()
            .map(|scope| scope.as_str().expect("a scope").to_owned())
            .collect();
        sqlx::query(
            "insert into esi_tokens (id, character_id, access_token, token_type, refresh_token,
                character_owner_hash, scopes, expires_at, created_at, updated_at)
             values ($1, $2, $3, 'Bearer', $4, $5, $6, $7, $8, $8)",
        )
        .bind(number)
        .bind(row["character_id"].as_i64())
        .bind(format!("access-token-{number}"))
        .bind(format!("refresh-token-{number}"))
        .bind(format!("owner-hash-{number}"))
        .bind(scopes)
        .bind(time_of(app, &row["expires_at"]))
        .bind(now)
        .execute(pool)
        .await
        .expect("a token");
    }
    for row in data["standings_sources"].as_array().expect("sources") {
        sqlx::query(
            "insert into standings_sources (type, entity_id, last_synced_at, created_at, updated_at)
             values ($1, $2, $3, $4, $4)",
        )
        .bind(row["type"].as_str())
        .bind(row["entity_id"].as_i64())
        .bind(time_of(app, &row["last_synced_at"]))
        .bind(now)
        .execute(pool)
        .await
        .expect("a source");
    }
    for row in data["source_contacts"].as_array().expect("contacts") {
        sqlx::query(
            "insert into source_contacts (contact_id, contact_type, standing, created_at, updated_at)
             values ($1, $2, $3::float8::numeric(4, 2), $4, $4)",
        )
        .bind(row["contact_id"].as_i64())
        .bind(row["contact_type"].as_str())
        .bind(row["standing"].as_f64())
        .bind(now)
        .execute(pool)
        .await
        .expect("a source contact");
    }
    for row in data["character_synced_contacts"]
        .as_array()
        .expect("ledger")
    {
        sqlx::query(
            "insert into character_synced_contacts (character_id, contact_id, standing, created_at,
                updated_at)
             values ($1, $2, $3::float8::numeric(4, 2), $4, $4)",
        )
        .bind(row["character_id"].as_i64())
        .bind(row["contact_id"].as_i64())
        .bind(row["standing"].as_f64())
        .bind(now)
        .execute(pool)
        .await
        .expect("a synced contact");
    }
    for row in data["standing_requests"].as_array().expect("requests") {
        sqlx::query(
            "insert into standing_requests (id, subject_type, subject_id, requested_by_character_id,
                status, created_at, updated_at)
             values ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(row["id"].as_i64())
        .bind(row["subject_type"].as_str())
        .bind(row["subject_id"].as_i64())
        .bind(row["requested_by_character_id"].as_i64())
        .bind(row["status"].as_str())
        .bind(time_of(app, &row["created_at"]))
        .bind(now)
        .execute(pool)
        .await
        .expect("a standing request");
    }
    for row in data["discord_settings"].as_array().expect("discord") {
        // N7: the webhook is the mock.
        let webhook = row["webhook_url"]
            .as_str()
            .map(|url| url.replace("https://discord.test/webhook", &app.discord.webhook()));
        sqlx::query(
            "insert into discord_settings (webhook_url, role_id, created_at, updated_at)
             values ($1, $2, $3, $3)",
        )
        .bind(webhook)
        .bind(row["role_id"].as_str())
        .bind(now)
        .execute(pool)
        .await
        .expect("discord settings");
    }
    for table in ["users", "esi_tokens", "standing_requests"] {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "select setval(pg_get_serial_sequence('{table}', 'id'), (select max(id) from {table}))"
        )))
        .execute(pool)
        .await
        .expect("a sequence");
    }
}

/// N8: a scenario's MySQL statement as Postgres runs it, `{minutes:N}`
/// resolved against the clock.
pub fn translate_sql(statement: &str, app: &TestApp) -> Vec<String> {
    let at = |minutes: i64| {
        let moment = app.clock.now() + time::Duration::minutes(minutes);
        format!(
            "'{}'::timestamptz",
            moment.format(&Rfc3339).expect("a time")
        )
    };
    if statement.starts_with("insert into standing_requests") && statement.contains("96000000") {
        // 28 more pending requests, older than the dataset's, a second
        // apart.
        return vec![format!(
            "insert into standing_requests (subject_type, subject_id, requested_by_character_id,
                status, created_at, updated_at)
             select 'character', 96000000 + n, 95000010, 'pending',
                {} - make_interval(secs => n), {}
             from generate_series(0, 27) n",
            at(-500),
            at(-500)
        )];
    }
    if statement.starts_with("update standing_requests set created_at = date_sub") {
        return Vec::new();
    }
    if statement.starts_with("insert into corporations") && statement.contains("Filler Corp") {
        return vec![format!(
            "insert into corporations (id, name, ticker, created_at, updated_at)
             select 97000000 + n, 'Filler Corp ' || lpad(n::text, 2, '0'), 'FILL', {0}, {0}
             from generate_series(0, 21) n",
            at(0)
        )];
    }
    if statement.starts_with("insert into characters") && statement.contains("Filler Pilot") {
        return vec![format!(
            "insert into characters (id, name, corporation_id, user_id, should_sync, created_at, updated_at)
             select 96500000 + n, 'Filler Pilot ' || lpad(n::text, 2, '0'), 97000000 + n, 4, false, {0}, {0}
             from generate_series(0, 21) n",
            at(0)
        )];
    }
    if statement.starts_with("delete from esi_token_scope") {
        // The admin's token without the mail scope.
        return vec![
            "update esi_tokens set scopes = array_remove(scopes, 'esi-mail.send_mail.v1') where id = 1"
                .to_owned(),
        ];
    }
    let mut statement = statement.to_owned();
    let pattern = regex::Regex::new(r"\{minutes:(-?\d+)\}").expect("a pattern");
    while let Some(found) = pattern.captures(&statement) {
        let minutes: i64 = found[1].parse().expect("minutes");
        let whole = found[0].to_owned();
        statement = statement.replacen(&whole, &at(minutes), 1);
    }
    vec![statement]
}

/// The rows record.py dumps, from the test database.
pub async fn rows(app: &TestApp) -> Value {
    let queries = [
        (
            "users",
            "select id, name, main_character_id, setup_completed_at is not null as set_up
             from users order by id",
        ),
        (
            "characters",
            "select id, name, corporation_id, alliance_id, user_id, should_sync,
                character_owner_hash is not null as has_owner_hash
             from characters order by id",
        ),
        (
            "corporations",
            "select id, name, ticker, alliance_id, member_count from corporations order by id",
        ),
        (
            "alliances",
            "select id, name, ticker from alliances order by id",
        ),
        (
            "esi_tokens",
            "select character_id, access_token, refresh_token,
                array(select unnest(scopes) order by 1) as scopes
             from esi_tokens order by character_id, id",
        ),
        (
            "source_contacts",
            "select contact_id, contact_type, standing::float8 as standing
             from source_contacts order by contact_id",
        ),
        (
            "character_synced_contacts",
            "select character_id, contact_id, standing::float8 as standing
             from character_synced_contacts order by character_id, contact_id",
        ),
        (
            "standings_sources",
            "select type, entity_id, last_synced_at is not null as synced
             from standings_sources order by id",
        ),
        (
            "standing_requests",
            "select id, subject_type, subject_id, requested_by_character_id, status
             from standing_requests order by id",
        ),
        (
            "discord_settings",
            "select webhook_url, role_id from discord_settings order by id",
        ),
        (
            "throttles",
            "select key from throttles where expires_at > now() - interval '100 years' order by key",
        ),
    ];
    let mut tables = Map::new();
    for (table, query) in queries {
        tables.insert(table.to_owned(), Value::Array(app.rows(query).await));
    }
    let mut value = Value::Object(tables);
    // N7 backwards: the mock's webhook is the dataset's.
    replace_string(
        &mut value,
        &app.discord.webhook(),
        "https://discord.test/webhook",
    );
    value
}

fn replace_string(value: &mut Value, from: &str, to: &str) {
    match value {
        Value::String(text) => {
            if text.contains(from) {
                *text = text.replace(from, to);
            }
        }
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| replace_string(item, from, to)),
        Value::Object(map) => map
            .values_mut()
            .for_each(|item| replace_string(item, from, to)),
        _ => {}
    }
}

/// Compares recorded rows with the test database's, numbers by value (the
/// reference wrote `10` and `10.0` alike), keys by name; N6 for the hour of
/// the alert throttle.
#[track_caller]
pub fn assert_rows_eq(expected: &Value, actual: &Value, app: &TestApp) {
    let hour = app.clock.now();
    let key = format!(
        "standings:source-unreadable:{:04}{:02}{:02}{:02}",
        hour.year(),
        u8::from(hour.month()),
        hour.day(),
        hour.hour()
    );
    let mut expected = expected.clone();
    if let Some(throttles) = expected["throttles"].as_array_mut() {
        for row in throttles {
            if row["key"]
                .as_str()
                .is_some_and(|value| value.starts_with("standings:source-unreadable:"))
            {
                row["key"] = json!(key);
            }
        }
    }
    for (table, rows) in expected.as_object().expect("tables") {
        let actual_rows = &actual[table];
        assert_eq!(
            rows.as_array().map(Vec::len),
            actual_rows.as_array().map(Vec::len),
            "{table}: {actual_rows}"
        );
        for (expected_row, actual_row) in rows
            .as_array()
            .expect("rows")
            .iter()
            .zip(actual_rows.as_array().expect("rows"))
        {
            for (column, value) in expected_row.as_object().expect("a row") {
                let other = &actual_row[column];
                let same = match (value, other) {
                    (Value::Number(a), Value::Number(b)) => a.as_f64() == b.as_f64(),
                    _ => value == other,
                };
                assert!(
                    same,
                    "{table}.{column}: expected {value}, got {other}\n  row {actual_row}"
                );
            }
        }
    }
}

/// Sets up the mocks with a scenario's recorded fakes.
pub fn configure_fakes(app: &TestApp, fakes: &Value) {
    app.esi.strict();
    for (line, answer) in fakes.as_object().into_iter().flatten() {
        let (method, url) = line.split_once(' ').expect("METHOD url");
        let status = answer["status"].as_u64().unwrap_or(200) as u16;
        let body = answer.get("body").cloned().unwrap_or(Value::Null);
        if let Some(path) = url.strip_prefix("https://esi.evetech.net") {
            let headers = answer["headers"]
                .as_object()
                .into_iter()
                .flatten()
                .map(|(name, value)| (name.clone(), value.as_str().unwrap_or_default().to_owned()))
                .collect();
            let body = match body {
                // The reference's fake answered the string as the body.
                Value::String(text) => serde_json::from_str(&text).unwrap_or(Value::String(text)),
                other => other,
            };
            app.esi
                .exact(&format!("{method} {path}"), status, body, headers);
        } else if url.starts_with("https://login.eveonline.com") {
            app.sso.refresh_answer(body);
        } else if url.starts_with("https://discord.test") {
            if status >= 400 {
                app.discord.fail_with(status);
            }
        } else {
            panic!("no mock for {line}");
        }
    }
}

/// What the app sent, per service, in record.py's form (N5).
pub fn calls(app: &TestApp) -> Value {
    json!({
        "esi": app.esi.requests().iter().map(|call| call_json(call, "https://esi.evetech.net")).collect::<Vec<_>>(),
        "sso": app.sso.requests().iter().map(|call| call_json(call, "https://login.eveonline.com")).collect::<Vec<_>>(),
        "discord": app.discord.requests().iter().map(|call| call_json(call, "https://discord.test")).collect::<Vec<_>>(),
    })
}

fn call_json(call: &Recorded, host: &str) -> Value {
    let url = match &call.query {
        Some(query) => format!("{host}{}?{query}", call.path),
        None => format!("{host}{}", call.path),
    };
    let body = if call.body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&call.body).unwrap_or_else(|_| Value::String(call.text()))
    };
    let mut headers = Map::new();
    for name in [
        "user-agent",
        "x-compatibility-date",
        "authorization",
        "content-type",
    ] {
        if let Some(value) = call.header(name) {
            headers.insert(name.to_owned(), json!(value));
        }
    }
    json!({ "method": call.method.as_str(), "url": url, "headers": headers, "body": body })
}

/// The recorded calls, split per service like `calls`, with the user
/// agent the app sends now (N5).
pub fn expected_calls(golden: &Value, user_agent: &str) -> Value {
    let mut esi = Vec::new();
    let mut sso = Vec::new();
    let mut discord = Vec::new();
    for call in golden["http"].as_array().into_iter().flatten() {
        let mut call = call.clone();
        let url = call["url"].as_str().unwrap_or_default().to_owned();
        let headers = call["headers"].as_object_mut().expect("headers");
        headers.shift_remove("host");
        if url.starts_with("https://discord.test") {
            headers.shift_remove("user-agent");
            discord.push(call);
        } else if url.starts_with("https://login.eveonline.com") {
            headers.insert("user-agent".to_owned(), json!(user_agent));
            sso.push(call);
        } else {
            headers.insert("user-agent".to_owned(), json!(user_agent));
            esi.push(call);
        }
    }
    json!({ "esi": esi, "sso": sso, "discord": discord })
}

/// The actual calls with Discord's user agent left out (N5).
pub fn comparable_calls(app: &TestApp) -> Value {
    let mut calls = calls(app);
    for call in calls["discord"].as_array_mut().expect("discord") {
        call["headers"]
            .as_object_mut()
            .expect("headers")
            .shift_remove("user-agent");
    }
    calls
}

/// The queued jobs as record.py names them: the kind and its payload.
pub async fn jobs(app: &TestApp) -> Value {
    let rows: Vec<(String, sqlx::types::Json<Value>)> =
        sqlx::query_as("select kind, payload from jobs where failed_at is null order by id")
            .fetch_all(&app.pool)
            .await
            .expect("jobs");
    Value::Array(
        rows.into_iter()
            .map(|(kind, payload)| {
                let mut job = Map::new();
                job.insert("kind".to_owned(), json!(kind));
                for (key, value) in payload.0.as_object().cloned().unwrap_or_default() {
                    job.insert(key, value);
                }
                Value::Object(job)
            })
            .collect(),
    )
}

pub async fn failed_jobs(app: &TestApp) -> i64 {
    sqlx::query_scalar("select count(*) from jobs where failed_at is not null")
        .fetch_one(&app.pool)
        .await
        .expect("failed jobs")
}

/// N4: the status, title and message of an HTML error page.
pub fn error_page_text(html: &str) -> (String, String, String) {
    let title = regex::Regex::new(r"<title>(.*?)</title>")
        .expect("a pattern")
        .captures(html)
        .map(|found| found[1].trim().to_owned())
        .unwrap_or_default();
    let found = regex::Regex::new(r"(?s)<h1[^>]*>\s*(.*?)\s*</h1>\s*<div[^>]*>\s*(.*?)\s*</div>")
        .expect("a pattern")
        .captures(html);
    match found {
        Some(found) => (title, found[1].to_owned(), found[2].to_owned()),
        None => (title, String::new(), String::new()),
    }
}

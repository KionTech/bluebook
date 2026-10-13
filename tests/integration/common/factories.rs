//! Factories that write rows straight to the test database. Ids and names
//! come from a counter, so runs are repeatable; tests that depend on a
//! name or id pass their own.

use std::sync::atomic::{AtomicI64, Ordering};

use serde_json::Value;
use time::OffsetDateTime;

use super::TestApp;

static SEQUENCE: AtomicI64 = AtomicI64::new(1);

fn next() -> i64 {
    SEQUENCE.fetch_add(1, Ordering::Relaxed)
}

/// Who owns a new character: a new user unless the test says otherwise.
#[derive(Debug, Clone, Copy, Default)]
pub enum Owner {
    #[default]
    NewUser,
    User(i64),
    Nobody,
}

/// A character to create; unset fields get defaults.
#[derive(Debug, Clone, Default)]
pub struct NewCharacter<'a> {
    pub id: Option<i64>,
    pub name: Option<&'a str>,
    pub owner: Owner,
    pub corporation_id: Option<i64>,
    pub alliance_id: Option<i64>,
    pub should_sync: bool,
}

/// A standing request to create; unset fields get defaults.
#[derive(Debug, Clone, Default)]
pub struct NewRequest<'a> {
    /// `character` unless given.
    pub subject_type: Option<&'a str>,
    /// The requesting character's id unless given.
    pub subject_id: Option<i64>,
    /// A new character (with a new user) unless given.
    pub requested_by: Option<i64>,
    /// `pending` unless given.
    pub status: Option<&'a str>,
    pub created_at: Option<OffsetDateTime>,
}

impl TestApp {
    /// A user who has been through the setup.
    pub async fn user(&self) -> i64 {
        self.user_named(&format!("User {}", next()), true).await
    }

    /// A user who has not been through the setup.
    pub async fn user_without_setup(&self) -> i64 {
        self.user_named(&format!("User {}", next()), false).await
    }

    pub async fn user_named(&self, name: &str, set_up: bool) -> i64 {
        let now = self.clock.now();
        sqlx::query_scalar(
            "insert into users (name, setup_completed_at, created_at, updated_at)
             values ($1, $2, $3, $3) returning id",
        )
        .bind(name)
        .bind(set_up.then_some(now))
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .expect("a user")
    }

    pub async fn character(&self, new: NewCharacter<'_>) -> i64 {
        let number = next();
        let id = new.id.unwrap_or(90_000_000 + number);
        let name = new
            .name
            .map_or_else(|| format!("Pilot {number:04}"), str::to_owned);
        let user_id = match new.owner {
            Owner::NewUser => Some(self.user().await),
            Owner::User(id) => Some(id),
            Owner::Nobody => None,
        };
        let now = self.clock.now();
        sqlx::query(
            "insert into characters (id, name, corporation_id, alliance_id, user_id,
                character_owner_hash, should_sync, created_at, updated_at)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $8)",
        )
        .bind(id)
        .bind(name)
        .bind(new.corporation_id)
        .bind(new.alliance_id)
        .bind(user_id)
        .bind(format!("owner-hash-{number}"))
        .bind(new.should_sync)
        .bind(now)
        .execute(&self.pool)
        .await
        .expect("a character");
        id
    }

    /// A character on the user's account.
    pub async fn character_of(&self, user_id: i64) -> i64 {
        self.character(NewCharacter {
            owner: Owner::User(user_id),
            ..NewCharacter::default()
        })
        .await
    }

    /// `$user->mainCharacter()->associate($character)->save()`.
    pub async fn set_main(&self, user_id: i64, character_id: i64) {
        sqlx::query("update users set main_character_id = $2 where id = $1")
            .bind(user_id)
            .bind(character_id)
            .execute(&self.pool)
            .await
            .expect("a main character");
    }

    /// `grantScopes($character, ...$scopes)`: a token carrying the scopes,
    /// valid for an hour.
    pub async fn grant_scopes(&self, character_id: i64, scopes: &[&str]) -> i64 {
        self.token(
            character_id,
            scopes,
            self.clock.now() + time::Duration::hours(1),
        )
        .await
    }

    pub async fn token(
        &self,
        character_id: i64,
        scopes: &[&str],
        expires_at: OffsetDateTime,
    ) -> i64 {
        let number = next();
        let now = self.clock.now();
        let scopes: Vec<String> = scopes.iter().map(|scope| (*scope).to_owned()).collect();
        sqlx::query_scalar(
            "insert into esi_tokens (character_id, access_token, token_type, refresh_token,
                character_owner_hash, scopes, expires_at, created_at, updated_at)
             values ($1, $2, 'Bearer', $3, $4, $5, $6, $7, $7) returning id",
        )
        .bind(character_id)
        .bind(format!("access-token-{number}"))
        .bind(format!("refresh-token-{number}"))
        .bind(format!("owner-hash-{number}"))
        .bind(scopes)
        .bind(expires_at)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .expect("a token")
    }

    /// `Corporation::query()->create([...])`.
    pub async fn corporation(
        &self,
        id: i64,
        name: Option<&str>,
        ticker: Option<&str>,
        alliance_id: Option<i64>,
    ) {
        let now = self.clock.now();
        sqlx::query(
            "insert into corporations (id, name, ticker, alliance_id, created_at, updated_at)
             values ($1, $2, $3, $4, $5, $5)
             on conflict (id) do nothing",
        )
        .bind(id)
        .bind(name)
        .bind(ticker)
        .bind(alliance_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .expect("a corporation");
    }

    /// `Alliance::query()->create([...])`.
    pub async fn alliance(&self, id: i64, name: Option<&str>, ticker: Option<&str>) {
        let now = self.clock.now();
        sqlx::query(
            "insert into alliances (id, name, ticker, created_at, updated_at)
             values ($1, $2, $3, $4, $4)
             on conflict (id) do nothing",
        )
        .bind(id)
        .bind(name)
        .bind(ticker)
        .bind(now)
        .execute(&self.pool)
        .await
        .expect("an alliance");
    }

    pub async fn faction(&self, id: i64, name: &str) {
        sqlx::query("insert into factions (id, name) values ($1, $2)")
            .bind(id)
            .bind(name)
            .execute(&self.pool)
            .await
            .expect("a faction");
    }

    /// `StandingsSource::create([...])`.
    pub async fn source(&self, kind: &str, entity_id: i64) {
        self.source_synced(kind, entity_id, None).await;
    }

    pub async fn source_synced(
        &self,
        kind: &str,
        entity_id: i64,
        last_synced_at: Option<OffsetDateTime>,
    ) {
        let now = self.clock.now();
        sqlx::query(
            "insert into standings_sources (type, entity_id, last_synced_at, created_at, updated_at)
             values ($1, $2, $3, $4, $4)",
        )
        .bind(kind)
        .bind(entity_id)
        .bind(last_synced_at)
        .bind(now)
        .execute(&self.pool)
        .await
        .expect("a source");
    }

    /// A contact of the source.
    pub async fn source_contact(&self, contact_id: i64, contact_type: &str, standing: f64) {
        let now = self.clock.now();
        sqlx::query(
            "insert into source_contacts (contact_id, contact_type, standing, created_at, updated_at)
             values ($1, $2, $3::float8::numeric(4, 2), $4, $4)",
        )
        .bind(contact_id)
        .bind(contact_type)
        .bind(standing)
        .bind(now)
        .execute(&self.pool)
        .await
        .expect("a source contact");
    }

    /// A contact of the source with any type and standing.
    pub async fn any_source_contact(&self) -> i64 {
        let number = next();
        let types = ["character", "corporation", "alliance", "faction"];
        let standings = [-10.0, -5.0, 0.0, 5.0, 10.0];
        let id = 90_000_000 + number;
        self.source_contact(
            id,
            types[(number % 4) as usize],
            standings[(number % 5) as usize],
        )
        .await;
        id
    }

    /// A ledger row: a contact the sync wrote to the character.
    pub async fn synced_contact(&self, character_id: i64, contact_id: i64, standing: f64) {
        let now = self.clock.now();
        sqlx::query(
            "insert into character_synced_contacts (character_id, contact_id, standing, created_at, updated_at)
             values ($1, $2, $3::float8::numeric(4, 2), $4, $4)",
        )
        .bind(character_id)
        .bind(contact_id)
        .bind(standing)
        .bind(now)
        .execute(&self.pool)
        .await
        .expect("a synced contact");
    }

    pub async fn standing_request(&self, new: NewRequest<'_>) -> i64 {
        let requested_by = match new.requested_by {
            Some(id) => id,
            None => self.character(NewCharacter::default()).await,
        };
        let now = self.clock.now();
        sqlx::query_scalar(
            "insert into standing_requests (subject_type, subject_id, requested_by_character_id,
                status, created_at, updated_at)
             values ($1, $2, $3, $4, $5, $5) returning id",
        )
        .bind(new.subject_type.unwrap_or("character"))
        .bind(new.subject_id.unwrap_or(requested_by))
        .bind(requested_by)
        .bind(new.status.unwrap_or("pending"))
        .bind(new.created_at.unwrap_or(now))
        .fetch_one(&self.pool)
        .await
        .expect("a standing request")
    }

    /// `DiscordSetting::query()->create([...])`.
    pub async fn discord_settings(&self, webhook_url: Option<&str>, role_id: Option<&str>) {
        let now = self.clock.now();
        sqlx::query(
            "insert into discord_settings (webhook_url, role_id, created_at, updated_at)
             values ($1, $2, $3, $3)",
        )
        .bind(webhook_url)
        .bind(role_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .expect("discord settings");
    }

    // Reading rows back.

    pub async fn scalar(&self, sql: &str) -> Value {
        let value: Option<sqlx::types::Json<Value>> = sqlx::query_scalar(sqlx::AssertSqlSafe(
            format!("select to_json(x.v) from ({sql}) x(v)"),
        ))
        .fetch_optional(&self.pool)
        .await
        .expect("a value");
        value.map_or(Value::Null, |value| value.0)
    }

    /// The rows of a query as JSON objects.
    pub async fn rows(&self, sql: &str) -> Vec<Value> {
        let rows: Vec<sqlx::types::Json<Value>> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "select to_json(x) from ({sql}) x"
        )))
        .fetch_all(&self.pool)
        .await
        .expect("rows");
        rows.into_iter().map(|row| row.0).collect()
    }

    pub async fn main_character_of(&self, user_id: i64) -> Option<i64> {
        sqlx::query_scalar("select main_character_id from users where id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .expect("a user")
    }

    pub async fn user_exists(&self, user_id: i64) -> bool {
        sqlx::query_scalar("select exists (select 1 from users where id = $1)")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .expect("a lookup")
    }

    pub async fn owner_of(&self, character_id: i64) -> Option<i64> {
        sqlx::query_scalar("select user_id from characters where id = $1")
            .bind(character_id)
            .fetch_one(&self.pool)
            .await
            .expect("a character")
    }

    pub async fn should_sync(&self, character_id: i64) -> bool {
        sqlx::query_scalar("select should_sync from characters where id = $1")
            .bind(character_id)
            .fetch_one(&self.pool)
            .await
            .expect("a character")
    }

    pub async fn request_status(&self, id: i64) -> String {
        sqlx::query_scalar("select status from standing_requests where id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .expect("a request")
    }
}

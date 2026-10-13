//! The standings: the one source (`StandingsSource`), its contacts as the
//! canonical standings (`SourceContact`), the refresh from ESI
//! (`source`), the five-minute command (`sync`), the sync of one
//! character's in-game contacts (`character`), the lapsed token mail
//! (`tokens`), and the standing that applies to an entity (`effective`).

pub mod character;
pub mod effective;
pub mod source;
pub mod sync;
pub mod tokens;

use sqlx::PgPool;
use time::OffsetDateTime;

use crate::scopes;

/// Standings are equal within this tolerance.
pub const STANDING_EPSILON: f64 = 0.01;

/// `"type:id"`, the key entities are looked up by.
pub fn key(kind: &str, id: i64) -> String {
    format!("{kind}:{id}")
}

/// SQL: the name of the subject of the standing request `r` asked through
/// the character `c`: the character's own name (a character asks for
/// itself), else the corporation or alliance by the subject's id. Laravel
/// took those from the requester's current corporation or alliance, which
/// named the wrong entity (or none) once the requester moved (FIXES.md,
/// "Requests named the requester's corporation, not the one asked for").
pub const SUBJECT_NAME_SQL: &str = "case r.subject_type
        when 'character' then c.name
        when 'corporation' then (select s.name from corporations s where s.id = r.subject_id)
        else (select s.name from alliances s where s.id = r.subject_id)
    end";

/// The configured standings source.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Source {
    #[sqlx(rename = "type")]
    pub kind: String,
    pub entity_id: i64,
    pub last_synced_at: Option<OffsetDateTime>,
    /// The last read was refused (403) and none worked since.
    pub unreadable_since: Option<OffsetDateTime>,
}

impl Source {
    /// `StandingsSource::current()`: the first row.
    pub async fn current(pool: &PgPool) -> sqlx::Result<Option<Source>> {
        sqlx::query_as(
            "select type, entity_id, last_synced_at, unreadable_since from standings_sources
             order by id limit 1",
        )
        .fetch_optional(pool)
        .await
    }

    /// Whether a character inherits the source in game: it is in the
    /// source corporation or alliance. A character source covers nobody.
    pub fn covers(&self, corporation_id: Option<i64>, alliance_id: Option<i64>) -> bool {
        match self.kind.as_str() {
            "corporation" => corporation_id == Some(self.entity_id),
            "alliance" => alliance_id == Some(self.entity_id),
            _ => false,
        }
    }

    /// The scope a character needs to read this source's contacts.
    pub fn required_scope(&self) -> &'static str {
        required_scope(&self.kind)
    }

    /// The display name of the source entity, from the local tables.
    pub async fn entity_name(&self, pool: &PgPool) -> sqlx::Result<Option<String>> {
        entity_name(pool, &self.kind, self.entity_id).await
    }

    /// Whether the source has no standing toward an entity yet, so it
    /// could be requested. The source entity itself never needs one.
    pub async fn requires_standing_for(
        &self,
        pool: &PgPool,
        kind: &str,
        id: i64,
    ) -> sqlx::Result<bool> {
        if kind == self.kind && id == self.entity_id {
            return Ok(false);
        }
        let exists: bool = sqlx::query_scalar(
            "select exists (select 1 from source_contacts where contact_type = $1 and contact_id = $2)",
        )
        .bind(kind)
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(!exists)
    }

    /// The summary the admin pages show (`StandingsSourceSummaryResource`).
    pub async fn summary(&self, pool: &PgPool) -> sqlx::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": self.kind,
            "entity_id": self.entity_id,
            "entity_name": self.entity_name(pool).await?,
            "last_synced_at": self.last_synced_at.map(crate::format::iso8601),
        }))
    }
}

pub fn required_scope(kind: &str) -> &'static str {
    match kind {
        "corporation" => scopes::READ_CORPORATION_CONTACTS,
        "alliance" => scopes::READ_ALLIANCE_CONTACTS,
        _ => scopes::READ_CHARACTER_CONTACTS,
    }
}

/// The name of a character, corporation, alliance or faction.
pub async fn entity_name(pool: &PgPool, kind: &str, id: i64) -> sqlx::Result<Option<String>> {
    let table = match kind {
        "character" => "characters",
        "corporation" => "corporations",
        "alliance" => "alliances",
        "faction" => "factions",
        _ => return Ok(None),
    };
    let name: Option<Option<String>> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select name from {table} where id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(name.flatten())
}

/// A source contact with its locally known entity: the name, and the
/// corporation and alliance a character entity is in, or the alliance of a
/// corporation entity.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Contact {
    pub contact_id: i64,
    pub contact_type: String,
    pub standing: f64,
    pub name: Option<String>,
    /// The entity is stored locally (Laravel's loaded `entity`).
    pub known: bool,
    pub parent_corporation_id: Option<i64>,
    pub parent_alliance_id: Option<i64>,
}

impl Contact {
    pub fn key(&self) -> String {
        key(&self.contact_type, self.contact_id)
    }

    /// Whether this contact's standing makes `other`'s redundant: an equal
    /// standing does, and a stronger one in the same direction (a +10
    /// parent covers a +5 child, a -10 parent a -5 child).
    pub fn covers_standing(&self, other: &Contact) -> bool {
        if (self.standing - other.standing).abs() < STANDING_EPSILON {
            return true;
        }
        if other.standing > 0.0 {
            return self.standing > other.standing;
        }
        if other.standing < 0.0 {
            return self.standing < other.standing;
        }
        false
    }
}

/// The source contacts with their entities, in `order` (a SQL order by).
pub async fn contacts(pool: &PgPool, order: &str) -> sqlx::Result<Vec<Contact>> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select c.contact_id, c.contact_type, c.standing::float8 as standing,
            case c.contact_type
                when 'character' then ch.name
                when 'corporation' then co.name
                when 'alliance' then al.name
                when 'faction' then f.name
            end as name,
            coalesce(ch.id, co.id, al.id, f.id) is not null as known,
            ch.corporation_id as parent_corporation_id,
            coalesce(ch.alliance_id, co.alliance_id) as parent_alliance_id
         from source_contacts c
         left join characters ch on c.contact_type = 'character' and ch.id = c.contact_id
         left join corporations co on c.contact_type = 'corporation' and co.id = c.contact_id
         left join alliances al on c.contact_type = 'alliance' and al.id = c.contact_id
         left join factions f on c.contact_type = 'faction' and f.id = c.contact_id
         order by {order}"
    )))
    .fetch_all(pool)
    .await
}

/// The characters that sync (opted in, with a write token), by id:
/// (id, corporation, alliance). Only `user_id`'s with one given.
pub async fn syncable(
    pool: &PgPool,
    user_id: Option<i64>,
) -> sqlx::Result<Vec<(i64, Option<i64>, Option<i64>)>> {
    sqlx::query_as(
        "select c.id, c.corporation_id, c.alliance_id from characters c
         where c.should_sync and ($1::bigint is null or c.user_id = $1)
            and exists (select 1 from esi_tokens t where t.character_id = c.id and $2 = any(t.scopes))
         order by c.id",
    )
    .bind(user_id)
    .bind(scopes::WRITE_CHARACTER_CONTACTS)
    .fetch_all(pool)
    .await
}

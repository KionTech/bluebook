//! The local demo (DEMO_ENABLED, never in production): the demo pilot the
//! demo login signs in as, and `seed_demo`'s sample data around it, so the
//! site can be tried without EVE SSO. The demo pilot's character
//! (DEMO_CHARACTER_ID) is the stack's admin character. No tokens and no
//! webhook are seeded, so nothing reaches EVE or Discord. The seed
//! replaces its own rows and can run again.

use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

pub const DEMO_PILOT: &str = "Demo Pilot";

const ALLIANCES: [(i64, &str, &str); 3] = [
    (99_990_001, "Demo Home Alliance", "HOME"),
    (99_990_002, "Blue Friends", "BLUE"),
    (99_990_003, "Red Menace", "RED"),
];

const CORPORATIONS: [(i64, &str, &str, Option<i64>); 4] = [
    (98_990_001, "Demo Corp", "DEMO", Some(99_990_001)),
    (98_990_002, "Friendly Corp", "FRND", Some(99_990_002)),
    (98_990_003, "Hostile Industries", "HSTL", Some(99_990_003)),
    (98_990_004, "Neutral Traders", "NTRL", None),
];

/// The user owning a character, created with the character when missing.
async fn owner(
    pool: &PgPool,
    character_id: i64,
    name: &str,
    corporation_id: Option<i64>,
    alliance_id: Option<i64>,
    set_up: bool,
    now: OffsetDateTime,
) -> sqlx::Result<i64> {
    let existing: Option<Option<i64>> =
        sqlx::query_scalar("select user_id from characters where id = $1")
            .bind(character_id)
            .fetch_optional(pool)
            .await?;
    let user_id = match existing.flatten() {
        Some(user_id) => user_id,
        None => {
            sqlx::query_scalar(
                "insert into users (name, setup_completed_at, created_at, updated_at)
                 values ($1, $2, $3, $3) returning id",
            )
            .bind(name)
            .bind(set_up.then_some(now))
            .bind(now)
            .fetch_one(pool)
            .await?
        }
    };
    character(
        pool,
        character_id,
        name,
        Some(user_id),
        corporation_id,
        alliance_id,
        now,
    )
    .await?;
    Ok(user_id)
}

async fn character(
    pool: &PgPool,
    id: i64,
    name: &str,
    user_id: Option<i64>,
    corporation_id: Option<i64>,
    alliance_id: Option<i64>,
    now: OffsetDateTime,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into characters (id, name, user_id, corporation_id, alliance_id, character_owner_hash,
            created_at, updated_at)
         values ($1, $2, $3, $4, $5, case when $3::bigint is null then null else 'demo' end, $6, $6)
         on conflict (id) do update set name = excluded.name, user_id = excluded.user_id,
            corporation_id = excluded.corporation_id, alliance_id = excluded.alliance_id",
    )
    .bind(id)
    .bind(name)
    .bind(user_id)
    .bind(corporation_id)
    .bind(alliance_id)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

/// The demo pilot's user, created (with its character) when missing; a
/// seeded pilot is left as the seed made it.
pub async fn pilot(pool: &PgPool, now: OffsetDateTime, character_id: i64) -> sqlx::Result<i64> {
    let existing: Option<Option<i64>> =
        sqlx::query_scalar("select user_id from characters where id = $1")
            .bind(character_id)
            .fetch_optional(pool)
            .await?;
    match existing.flatten() {
        Some(user_id) => Ok(user_id),
        None => owner(pool, character_id, DEMO_PILOT, None, None, true, now).await,
    }
}

/// The demo pilot in an alliance that is the source, an alt that needs a
/// token, a friend, a new pilot without a main, the source's standings
/// and a few requests.
pub async fn seed(pool: &PgPool, now: OffsetDateTime, character_id: i64) -> sqlx::Result<()> {
    for (id, name, ticker) in ALLIANCES {
        sqlx::query(
            "insert into alliances (id, name, ticker, created_at, updated_at) values ($1, $2, $3, $4, $4)
             on conflict (id) do update set name = excluded.name, ticker = excluded.ticker",
        )
        .bind(id)
        .bind(name)
        .bind(ticker)
        .bind(now)
        .execute(pool)
        .await?;
    }
    for (id, name, ticker, alliance_id) in CORPORATIONS {
        sqlx::query(
            "insert into corporations (id, name, ticker, alliance_id, member_count, created_at, updated_at)
             values ($1, $2, $3, $4, 12, $5, $5)
             on conflict (id) do update set name = excluded.name, ticker = excluded.ticker,
                alliance_id = excluded.alliance_id",
        )
        .bind(id)
        .bind(name)
        .bind(ticker)
        .bind(alliance_id)
        .bind(now)
        .execute(pool)
        .await?;
    }
    sqlx::query(
        "insert into factions (id, name, created_at, updated_at) values (500001, 'Caldari State', $1, $1)
         on conflict (id) do nothing",
    )
    .bind(now)
    .execute(pool)
    .await?;

    let demo = owner(
        pool,
        character_id,
        DEMO_PILOT,
        Some(98_990_001),
        Some(99_990_001),
        true,
        now,
    )
    .await?;
    character(
        pool,
        character_id + 1,
        "Demo Alt",
        Some(demo),
        Some(98_990_004),
        None,
        now,
    )
    .await?;
    sqlx::query("update characters set should_sync = true where id = $1")
        .bind(character_id + 1)
        .execute(pool)
        .await?;
    sqlx::query("update users set main_character_id = $2 where id = $1")
        .bind(demo)
        .bind(character_id)
        .execute(pool)
        .await?;
    let friend = character_id + 2;
    owner(
        pool,
        friend,
        "Demo Friend",
        Some(98_990_002),
        Some(99_990_002),
        true,
        now,
    )
    .await?;
    let newbie = character_id + 3;
    owner(
        pool,
        newbie,
        "Demo Newbie",
        Some(98_990_004),
        None,
        false,
        now,
    )
    .await?;
    let hostile = character_id + 4;
    character(
        pool,
        hostile,
        "Demo Hostile",
        None,
        Some(98_990_003),
        Some(99_990_003),
        now,
    )
    .await?;

    sqlx::query("delete from standings_sources")
        .execute(pool)
        .await?;
    sqlx::query(
        "insert into standings_sources (type, entity_id, last_synced_at, created_at, updated_at)
         values ('alliance', 99990001, $1, $2, $2)",
    )
    .bind(now - Duration::minutes(2))
    .bind(now)
    .execute(pool)
    .await?;
    sqlx::query("delete from source_contacts")
        .execute(pool)
        .await?;
    for (id, kind, standing) in [
        (99_990_002, "alliance", 10.0),
        (98_990_002, "corporation", 10.0),
        (friend, "character", 5.0),
        (98_990_004, "corporation", 0.0),
        (500_001, "faction", -5.0),
        (99_990_003, "alliance", -10.0),
        (98_990_003, "corporation", -10.0),
        (hostile, "character", -10.0),
    ] {
        sqlx::query(
            "insert into source_contacts (contact_id, contact_type, standing, created_at, updated_at)
             values ($1, $2, $3::float8::numeric(4, 2), $4, $4)",
        )
        .bind(id)
        .bind(kind)
        .bind(standing)
        .bind(now)
        .execute(pool)
        .await?;
    }

    let demo_characters = vec![character_id, character_id + 1, friend, newbie, hostile];
    sqlx::query("delete from standing_requests where requested_by_character_id = any($1)")
        .bind(&demo_characters)
        .execute(pool)
        .await?;
    for (subject_type, subject_id, by, status, minutes) in [
        ("character", newbie, newbie, "pending", 20),
        ("corporation", 98_990_004, character_id + 1, "pending", 45),
        ("character", hostile, hostile, "rejected", 300),
        ("character", friend, friend, "done", 900),
    ] {
        sqlx::query(
            "insert into standing_requests (subject_type, subject_id, requested_by_character_id,
                status, created_at, updated_at)
             values ($1, $2, $3, $4, $5, $5)
             on conflict (subject_type, subject_id) do nothing",
        )
        .bind(subject_type)
        .bind(subject_id)
        .bind(by)
        .bind(status)
        .bind(now - Duration::minutes(minutes))
        .execute(pool)
        .await?;
    }
    Ok(())
}

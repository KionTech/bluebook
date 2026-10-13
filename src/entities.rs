//! The locally known EVE entities (characters, corporations, alliances),
//! created and refreshed from ESI's public endpoints, and the hourly
//! affiliation refresh.

use std::collections::HashMap;

use time::OffsetDateTime;

use crate::esi::{Affiliation, EsiError};
use crate::server::AppState;

/// ESI's bulk limit for affiliations and names.
pub const BULK_CHUNK: usize = 1000;

fn date(value: &Option<String>) -> Option<OffsetDateTime> {
    value.as_deref().and_then(|text| {
        OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339).ok()
    })
}

/// Makes sure an alliance exists locally with a name, fetching its
/// details the first time (or while its name is unknown).
pub async fn ensure_alliance(state: &AppState, id: i64) -> sqlx::Result<()> {
    let known: Option<Option<String>> =
        sqlx::query_scalar("select name from alliances where id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;
    if matches!(known, Some(Some(_))) {
        return Ok(());
    }
    let now = state.clock.now();
    let alliance = state.esi.alliance(id).await.ok();
    let (name, ticker, founded) = match &alliance {
        Some(alliance) => (
            alliance.name.clone(),
            alliance.ticker.clone(),
            date(&alliance.date_founded),
        ),
        None => (None, None, None),
    };
    sqlx::query(
        "insert into alliances (id, name, ticker, date_founded, created_at, updated_at)
         values ($1, $2, $3, $4, $5, $5)
         on conflict (id) do update set
            name = coalesce($6, alliances.name),
            ticker = coalesce($7, alliances.ticker),
            date_founded = coalesce($8, alliances.date_founded),
            updated_at = case when $9 then excluded.updated_at else alliances.updated_at end",
    )
    .bind(id)
    .bind(&name)
    .bind(&ticker)
    .bind(founded)
    .bind(now)
    .bind(&name)
    .bind(&ticker)
    .bind(founded)
    .bind(alliance.is_some())
    .execute(&state.pool)
    .await?;
    Ok(())
}

/// Makes sure a corporation exists locally with a name, fetching its
/// details the first time.
pub async fn ensure_corporation(state: &AppState, id: i64) -> sqlx::Result<()> {
    let known: Option<Option<String>> =
        sqlx::query_scalar("select name from corporations where id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;
    if matches!(known, Some(Some(_))) {
        return Ok(());
    }
    fill_corporation(state, id).await
}

/// Re-fetches a corporation even when it is known, keeping its alliance
/// membership current.
pub async fn refresh_corporation(state: &AppState, id: i64) -> sqlx::Result<()> {
    fill_corporation(state, id).await
}

async fn fill_corporation(state: &AppState, id: i64) -> sqlx::Result<()> {
    let now = state.clock.now();
    let Ok(corporation) = state.esi.corporation(id).await else {
        // The row exists afterwards either way.
        sqlx::query(
            "insert into corporations (id, created_at, updated_at) values ($1, $2, $2)
             on conflict (id) do nothing",
        )
        .bind(id)
        .bind(now)
        .execute(&state.pool)
        .await?;
        return Ok(());
    };
    let alliance_id = corporation.alliance_id.filter(|id| *id != 0);
    if let Some(alliance_id) = alliance_id {
        ensure_alliance(state, alliance_id).await?;
    }
    sqlx::query(
        "insert into corporations (id, name, ticker, member_count, tax_rate, war_eligible,
            date_founded, description, url, shares, alliance_id, created_at, updated_at)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)
         on conflict (id) do update set
            name = excluded.name, ticker = excluded.ticker,
            member_count = excluded.member_count, tax_rate = excluded.tax_rate,
            war_eligible = excluded.war_eligible, date_founded = excluded.date_founded,
            description = excluded.description, url = excluded.url, shares = excluded.shares,
            alliance_id = excluded.alliance_id, updated_at = excluded.updated_at",
    )
    .bind(id)
    .bind(&corporation.name)
    .bind(&corporation.ticker)
    .bind(corporation.member_count)
    .bind(corporation.tax_rate)
    .bind(corporation.war_eligible)
    .bind(date(&corporation.date_founded))
    .bind(&corporation.description)
    .bind(&corporation.url)
    .bind(corporation.shares)
    .bind(alliance_id)
    .bind(now)
    .execute(&state.pool)
    .await?;
    Ok(())
}

/// Makes sure an affiliation's alliance and corporation exist locally.
pub async fn ensure_affiliation(state: &AppState, affiliation: &Affiliation) -> sqlx::Result<()> {
    if let Some(alliance_id) = affiliation.alliance_id.filter(|id| *id != 0) {
        ensure_alliance(state, alliance_id).await?;
    }
    ensure_corporation(state, affiliation.corporation_id).await
}

/// Resolves the current corporation and alliance of characters in bulk
/// and stores them, creating the characters and their parents as needed.
/// A known name is stored with it; a character without one keeps its own.
/// Returns how many characters were stored.
pub async fn sync_character_affiliations(
    state: &AppState,
    ids: &[i64],
    names: &HashMap<i64, String>,
) -> Result<usize, sqlx::Error> {
    let mut unique: Vec<i64> = Vec::new();
    for id in ids {
        if !unique.contains(id) {
            unique.push(*id);
        }
    }
    let mut synced = 0;
    for chunk in unique.chunks(BULK_CHUNK) {
        let affiliations = match state.esi.affiliations(chunk).await {
            Ok(affiliations) => affiliations,
            Err(error) => {
                tracing::warn!(%error, "affiliations not read");
                continue;
            }
        };
        for affiliation in &affiliations {
            ensure_affiliation(state, affiliation).await?;
        }
        let now = state.clock.now();
        for affiliation in &affiliations {
            let alliance_id = affiliation.alliance_id.filter(|id| *id != 0);
            sqlx::query(
                "insert into characters (id, name, corporation_id, alliance_id, created_at, updated_at)
                 values ($1, $2, $3, $4, $5, $5)
                 on conflict (id) do update set
                    name = coalesce($2, characters.name),
                    corporation_id = excluded.corporation_id,
                    alliance_id = excluded.alliance_id,
                    updated_at = excluded.updated_at",
            )
            .bind(affiliation.character_id)
            .bind(names.get(&affiliation.character_id))
            .bind(affiliation.corporation_id)
            .bind(alliance_id)
            .bind(now)
            .execute(&state.pool)
            .await?;
            synced += 1;
        }
    }
    Ok(synced)
}

/// `refreshEntities`: corporations created (or, with
/// `refresh_corporations`, re-fetched), alliances created, then the
/// characters' affiliations.
pub async fn refresh_entities(
    state: &AppState,
    character_ids: &[i64],
    corporation_ids: &[i64],
    alliance_ids: &[i64],
    names: &HashMap<i64, String>,
    refresh_corporations: bool,
) -> Result<usize, sqlx::Error> {
    for id in corporation_ids {
        if refresh_corporations {
            refresh_corporation(state, *id).await?;
        } else {
            ensure_corporation(state, *id).await?;
        }
    }
    for id in alliance_ids {
        ensure_alliance(state, *id).await?;
    }
    sync_character_affiliations(state, character_ids, names).await
}

/// Resolves the names of ids in bulk; ids ESI does not answer are left
/// out.
pub async fn resolve_names(state: &AppState, ids: &[i64]) -> HashMap<i64, String> {
    let mut names = HashMap::new();
    for chunk in ids.chunks(BULK_CHUNK) {
        match state.esi.names(chunk).await {
            Ok(answer) => {
                for name in answer {
                    names.insert(name.id, name.name);
                }
            }
            Err(EsiError::Status(status)) => tracing::warn!(status, "names not resolved"),
            Err(error) => tracing::warn!(%error, "names not resolved"),
        }
    }
    names
}

/// `characters:sync-affiliations`: the characters on an account and the
/// source's character contacts get their affiliations, the source's
/// corporation contacts are re-fetched and its alliance contacts created.
/// Skipped during EVE's downtime. Returns the exit code.
pub async fn sync_affiliations(state: &AppState) -> i32 {
    if crate::downtime::is_active(state.clock.now()) {
        tracing::info!("Skipping affiliation sync during EVE downtime.");
        return 0;
    }
    match run_affiliations(state).await {
        Ok((characters, corporations)) => {
            tracing::info!(
                "Refreshed affiliations for {characters} character(s) and {corporations} corporation(s)."
            );
            0
        }
        Err(error) => {
            tracing::error!(%error, "affiliation sync failed");
            1
        }
    }
}

async fn run_affiliations(state: &AppState) -> sqlx::Result<(usize, usize)> {
    let contacts: Vec<(i64, String)> =
        sqlx::query_as("select contact_id, contact_type from source_contacts order by id")
            .fetch_all(&state.pool)
            .await?;
    let of_type = |kind: &str| -> Vec<i64> {
        contacts
            .iter()
            .filter(|(_, contact_type)| contact_type == kind)
            .map(|(id, _)| *id)
            .collect()
    };
    // By account, then id.
    let mut character_ids: Vec<i64> = sqlx::query_scalar(
        "select id from characters where user_id is not null order by user_id, id",
    )
    .fetch_all(&state.pool)
    .await?;
    for id in of_type("character") {
        if !character_ids.contains(&id) {
            character_ids.push(id);
        }
    }
    let corporations = of_type("corporation");
    let synced = refresh_entities(
        state,
        &character_ids,
        &corporations,
        &of_type("alliance"),
        &HashMap::new(),
        true,
    )
    .await?;
    Ok((synced, corporations.len()))
}

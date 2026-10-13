//! The source refresh: an admin character that
//! can read the source fetches its contacts; when they changed they
//! become the canonical standings, their entities are stored, and pending
//! requests that got a direct blue standing are done. A 403 (the reader
//! lacks the in-game role) marks the source unreadable until a read works.

use std::collections::HashMap;

use crate::entities;
use crate::esi;
use crate::server::AppState;
use crate::standings::{STANDING_EPSILON, Source};

/// The admin character that reads the source: one of the configured
/// admin characters holding the source's read scope and (for a
/// corporation or alliance) a member of it; the first by id.
pub async fn reader(state: &AppState, source: &Source) -> sqlx::Result<Option<i64>> {
    let member = match source.kind.as_str() {
        "corporation" => "c.corporation_id = $3",
        "alliance" => "c.alliance_id = $3",
        _ => "c.id = $3",
    };
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "select c.id from characters c
         where c.id = any($1) and {member}
            and exists (select 1 from esi_tokens t where t.character_id = c.id and $2 = any(t.scopes))
         order by c.id
         limit 1"
    )))
    .bind(&state.config.admin_character_ids)
    .bind(source.required_scope())
    .bind(source.entity_id)
    .fetch_optional(&state.pool)
    .await
}

/// Re-reads the source's contacts. `None` on failure (no source, no
/// reader, ESI failed), `Some(true)` when the standings changed,
/// `Some(false)` when they were up to date.
pub async fn refresh(state: &AppState) -> Option<bool> {
    match try_refresh(state).await {
        Ok(outcome) => outcome,
        Err(error) => {
            tracing::error!(%error, "source refresh failed");
            None
        }
    }
}

async fn try_refresh(state: &AppState) -> Result<Option<bool>, sqlx::Error> {
    let Some(source) = Source::current(&state.pool).await? else {
        return Ok(None);
    };
    let Some(reader) = reader(state, &source).await? else {
        return Ok(None);
    };
    let owner = match source.kind.as_str() {
        "corporation" => format!("corporations/{}", source.entity_id),
        "alliance" => format!("alliances/{}", source.entity_id),
        _ => format!("characters/{reader}"),
    };
    let contacts = match esi::contacts(state, reader, source.required_scope(), &owner).await {
        Ok(contacts) => contacts,
        Err(error) => {
            tracing::warn!(%error, "the source could not be read");
            if error.is_forbidden() {
                // The reader lacks the in-game role: the admins are told
                // (the shared `source_unreadable`, the Discord alert); the
                // token stays.
                sqlx::query(
                    "update standings_sources set unreadable_since = coalesce(unreadable_since, $1)",
                )
                .bind(state.clock.now())
                .execute(&state.pool)
                .await?;
            }
            return Ok(None);
        }
    };
    let changed = store_contacts(state, &contacts).await?;
    let now = state.clock.now();
    sqlx::query(
        "update standings_sources set last_synced_at = $1, updated_at = $1, unreadable_since = null",
    )
        .bind(now)
        .execute(&state.pool)
        .await?;
    Ok(Some(changed))
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Stores the contacts as the canonical standings; false (and nothing
/// done, no entity requests) when the ids and values are the same.
async fn store_contacts(state: &AppState, contacts: &[esi::Contact]) -> sqlx::Result<bool> {
    let mut incoming: Vec<(i64, f64)> = Vec::new();
    for contact in contacts {
        let standing = round2(contact.standing);
        match incoming
            .iter_mut()
            .find(|(id, _)| *id == contact.contact_id)
        {
            Some(entry) => entry.1 = standing,
            None => incoming.push((contact.contact_id, standing)),
        }
    }
    let existing: HashMap<i64, f64> =
        sqlx::query_as::<_, (i64, f64)>("select contact_id, standing::float8 from source_contacts")
            .fetch_all(&state.pool)
            .await?
            .into_iter()
            .map(|(id, standing)| (id, round2(standing)))
            .collect();
    let differs = existing.len() != incoming.len()
        || incoming.iter().any(|(id, standing)| {
            existing
                .get(id)
                .is_none_or(|known| (known - standing).abs() >= STANDING_EPSILON)
        });
    if !differs {
        return Ok(false);
    }

    let now = state.clock.now();
    let ids: Vec<i64> = incoming.iter().map(|(id, _)| *id).collect();
    let mut tx = state.pool.begin().await?;
    sqlx::query("delete from source_contacts where not (contact_id = any($1))")
        .bind(&ids)
        .execute(&mut *tx)
        .await?;
    for contact in contacts {
        sqlx::query(
            "insert into source_contacts (contact_id, contact_type, standing, created_at, updated_at)
             values ($1, $2, $3::float8::numeric(4, 2), $4, $4)
             on conflict (contact_id) do update set
                contact_type = excluded.contact_type, standing = excluded.standing,
                updated_at = excluded.updated_at",
        )
        .bind(contact.contact_id)
        .bind(&contact.contact_type)
        .bind(contact.standing)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    store_contact_entities(state, contacts).await?;
    mark_fulfilled_requests_done(state).await?;
    Ok(true)
}

/// The contacts' entities, stored so names and affiliations resolve
/// locally. Corporations are only created here; keeping their alliance
/// current is the hourly affiliation sync's job. Factions are static.
async fn store_contact_entities(state: &AppState, contacts: &[esi::Contact]) -> sqlx::Result<()> {
    let of_type = |kind: &str| -> Vec<i64> {
        contacts
            .iter()
            .filter(|contact| contact.contact_type == kind)
            .map(|contact| contact.contact_id)
            .collect()
    };
    let characters = of_type("character");
    let names = entities::resolve_names(state, &characters).await;
    entities::refresh_entities(
        state,
        &characters,
        &of_type("corporation"),
        &of_type("alliance"),
        &names,
        false,
    )
    .await?;
    Ok(())
}

/// Pending requests whose subject now has a direct, positive standing
/// are done. A standing inherited through a parent leaves the request to
/// an admin.
async fn mark_fulfilled_requests_done(state: &AppState) -> sqlx::Result<()> {
    sqlx::query(
        "update standing_requests r set status = 'done', updated_at = $1
         where r.status = 'pending' and exists (
            select 1 from source_contacts c
            where c.contact_type = r.subject_type and c.contact_id = r.subject_id and c.standing > 0
         )",
    )
    .bind(state.clock.now())
    .execute(&state.pool)
    .await?;
    Ok(())
}

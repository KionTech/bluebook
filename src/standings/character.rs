//! The sync of one character's in-game contacts with the source
//! (`CharacterStandingsSyncService`): new standings are added, changed ones
//! edited, and only contacts the sync wrote before and the source dropped
//! are deleted; personal contacts are never touched. The ledger
//! (`character_synced_contacts`) is then exactly the source's list, also
//! when ESI refused a write (FIXES.md, needs owner decision).

use std::collections::HashMap;

use reqwest::Method;

use crate::auth::tokens::has_scope;
use crate::esi;
use crate::format::php_float_string;
use crate::scopes::{READ_CHARACTER_CONTACTS, WRITE_CHARACTER_CONTACTS};
use crate::server::AppState;
use crate::standings::{STANDING_EPSILON, Source};

/// ESI takes up to 100 ids per add or edit and 20 per delete.
const WRITE_CHUNK: usize = 100;
const DELETE_CHUNK: usize = 20;

/// Ids grouped by standing, the groups in the order of their first id.
type ByStanding = Vec<(String, f64, Vec<i64>)>;

fn group(groups: &mut ByStanding, standing: f64, id: i64) {
    let label = php_float_string(standing);
    match groups.iter_mut().find(|(key, _, _)| *key == label) {
        Some((_, _, ids)) => ids.push(id),
        None => groups.push((label, standing, vec![id])),
    }
}

/// Syncs one character. A character that does not sync, has no write
/// token or inherits the source is left alone. An error only for a
/// character that does not exist or a database failure.
pub async fn sync(state: &AppState, character_id: i64) -> sqlx::Result<()> {
    let character: Option<(bool, Option<i64>, Option<i64>)> = sqlx::query_as(
        "select should_sync, corporation_id, alliance_id from characters where id = $1",
    )
    .bind(character_id)
    .fetch_optional(&state.pool)
    .await?;
    let Some((should_sync, corporation_id, alliance_id)) = character else {
        return Err(sqlx::Error::RowNotFound);
    };
    if !should_sync || !has_scope(&state.pool, character_id, WRITE_CHARACTER_CONTACTS).await? {
        return Ok(());
    }
    // Characters in the source corporation or alliance inherit it in game.
    if Source::current(&state.pool)
        .await?
        .is_some_and(|source| source.covers(corporation_id, alliance_id))
    {
        return Ok(());
    }

    let desired: Vec<(i64, f64)> =
        sqlx::query_as("select contact_id, standing::float8 from source_contacts order by id")
            .fetch_all(&state.pool)
            .await?;
    let current = current_contacts(state, character_id).await?;
    let ledger: Vec<i64> = sqlx::query_scalar(
        "select contact_id from character_synced_contacts where character_id = $1 order by id",
    )
    .bind(character_id)
    .fetch_all(&state.pool)
    .await?;

    let mut to_add: ByStanding = Vec::new();
    let mut to_edit: ByStanding = Vec::new();
    for (id, standing) in &desired {
        match current.get(id) {
            None => group(&mut to_add, *standing, *id),
            Some(now) if (now - standing).abs() >= STANDING_EPSILON => {
                group(&mut to_edit, *standing, *id);
            }
            Some(_) => {}
        }
    }
    let to_delete: Vec<i64> = ledger
        .into_iter()
        .filter(|id| !desired.iter().any(|(wanted, _)| wanted == id) && current.contains_key(id))
        .collect();

    // Write results are not looked at, as before.
    for (method, groups) in [(Method::POST, &to_add), (Method::PUT, &to_edit)] {
        for (_, standing, ids) in groups {
            for chunk in ids.chunks(WRITE_CHUNK) {
                if let Err(error) =
                    esi::write_contacts(state, character_id, method.clone(), chunk, *standing).await
                {
                    tracing::warn!(character_id, %error, "contacts not written");
                }
            }
        }
    }
    for chunk in to_delete.chunks(DELETE_CHUNK) {
        if let Err(error) = esi::delete_contacts(state, character_id, chunk).await {
            tracing::warn!(character_id, %error, "contacts not deleted");
        }
    }

    update_ledger(state, character_id, &desired).await
}

/// The character's contacts in game; none when it cannot read them.
async fn current_contacts(state: &AppState, character_id: i64) -> sqlx::Result<HashMap<i64, f64>> {
    if !has_scope(&state.pool, character_id, READ_CHARACTER_CONTACTS).await? {
        return Ok(HashMap::new());
    }
    match esi::contacts(
        state,
        character_id,
        READ_CHARACTER_CONTACTS,
        &format!("characters/{character_id}"),
    )
    .await
    {
        Ok(contacts) => Ok(contacts
            .into_iter()
            .map(|contact| (contact.contact_id, contact.standing))
            .collect()),
        Err(error) => {
            tracing::warn!(character_id, %error, "contacts not read");
            Ok(HashMap::new())
        }
    }
}

async fn update_ledger(
    state: &AppState,
    character_id: i64,
    desired: &[(i64, f64)],
) -> sqlx::Result<()> {
    let ids: Vec<i64> = desired.iter().map(|(id, _)| *id).collect();
    let now = state.clock.now();
    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "delete from character_synced_contacts where character_id = $1 and not (contact_id = any($2))",
    )
    .bind(character_id)
    .bind(&ids)
    .execute(&mut *tx)
    .await?;
    for (id, standing) in desired {
        sqlx::query(
            "insert into character_synced_contacts (character_id, contact_id, standing, created_at, updated_at)
             values ($1, $2, $3::float8::numeric(4, 2), $4, $4)
             on conflict (character_id, contact_id) do update set
                standing = excluded.standing, updated_at = excluded.updated_at",
        )
        .bind(character_id)
        .bind(id)
        .bind(standing)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

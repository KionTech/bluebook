//! The pilot's forms: switching sync per character, syncing now,
//! requesting a standing, the active and the main character, removing a
//! character, finishing the setup wizard, and `/settings`.

use axum::extract::{Path, State};
use axum::http::Uri;
use axum::response::{IntoResponse, Response};

use super::AppState;
use super::account::{self, Account};
use super::answer::{self, Done, Errors};
use super::errors;
use super::input::{Input, as_boolean};
use super::viewer::{self, Viewer};
use crate::auth::session;
use crate::jobs::{self, Job};
use crate::standings::Source;

/// Flashes a message and answers where to go (`None`: back).
async fn done_with(
    state: &AppState,
    viewer: &Viewer,
    redirect: Option<&str>,
    kind: &str,
    message: &str,
) -> Result<Response, Response> {
    let done = match redirect {
        Some(path) => Done::to(path),
        None => Done::back(),
    };
    let mut response = done.into_response();
    viewer::flash(state, viewer, kind, message, &mut response)
        .await
        .map_err(errors::server_error)?;
    Ok(response)
}

fn display(name: &Option<String>) -> &str {
    name.as_deref().unwrap_or_default()
}

/// `PUT /standings/{character}` with `should_sync`.
pub async fn update_sync(
    State(state): State<AppState>,
    viewer: Viewer,
    Path(id): Path<String>,
    input: Input,
) -> Result<Response, Response> {
    let account = account::form_account(&state, &viewer).await?;
    let character = account::owned_character(&state, &account, &id).await?;
    let mut errors_found = Errors::default();
    let should_sync = match input.get("should_sync") {
        None => {
            errors_found.add("should_sync", "The should sync field is required.");
            None
        }
        Some(value) => {
            let flag = as_boolean(value);
            if flag.is_none() {
                errors_found.add(
                    "should_sync",
                    "The should sync field must be true or false.",
                );
            }
            flag
        }
    };
    errors_found.finish()?;
    let should_sync = should_sync.expect("a validated flag");
    sqlx::query("update characters set should_sync = $2, updated_at = $3 where id = $1")
        .bind(character.id)
        .bind(should_sync)
        .bind(state.clock.now())
        .execute(&state.pool)
        .await
        .map_err(errors::server_error)?;
    let message = if should_sync {
        format!("{} will now sync standings.", display(&character.name))
    } else {
        format!(
            "{} will no longer sync standings.",
            display(&character.name)
        )
    };
    done_with(&state, &viewer, None, "success", &message).await
}

/// `POST /standings/sync`: a sync of every character of the account that
/// syncs and does not inherit the source.
pub async fn sync_now(State(state): State<AppState>, viewer: Viewer) -> Result<Response, Response> {
    let account = account::form_account(&state, &viewer).await?;
    let queued = crate::standings::sync::queue_character_syncs(&state, Some(account.id))
        .await
        .map_err(errors::server_error)?;
    if queued > 0 {
        done_with(
            &state,
            &viewer,
            None,
            "success",
            &format!("Re-syncing {queued} character(s)."),
        )
        .await
    } else {
        done_with(
            &state,
            &viewer,
            None,
            "info",
            "No characters are set to sync.",
        )
        .await
    }
}

/// `POST /standings/{character}/request` with `type`: a standing request
/// for the character, its corporation or its alliance.
pub async fn request_standing(
    State(state): State<AppState>,
    viewer: Viewer,
    Path(id): Path<String>,
    input: Input,
) -> Result<Response, Response> {
    let account = account::form_account(&state, &viewer).await?;
    let character = account::owned_character(&state, &account, &id).await?;
    let mut errors_found = Errors::default();
    let kind = match input.get("type") {
        None => {
            errors_found.add("type", "The type field is required.");
            None
        }
        Some(value) => match value
            .as_str()
            .filter(|kind| ["character", "corporation", "alliance"].contains(kind))
        {
            Some(kind) => Some(kind.to_owned()),
            None => {
                errors_found.add("type", "The selected type is invalid.");
                None
            }
        },
    };
    errors_found.finish()?;
    let kind = kind.expect("a validated type");
    let subject = match kind.as_str() {
        "character" => Some(character.id),
        "corporation" => character.corporation_id,
        _ => character.alliance_id,
    };
    let source = Source::current(&state.pool)
        .await
        .map_err(errors::server_error)?;
    let requestable = match (subject, &source) {
        (Some(subject), Some(source)) => source
            .requires_standing_for(&state.pool, &kind, subject)
            .await
            .map_err(errors::server_error)?,
        _ => false,
    };
    let Some(subject) = subject.filter(|_| requestable) else {
        return done_with(
            &state,
            &viewer,
            None,
            "error",
            "This standing cannot be requested.",
        )
        .await;
    };
    let existing: Option<String> = sqlx::query_scalar(
        "select status from standing_requests where subject_type = $1 and subject_id = $2",
    )
    .bind(&kind)
    .bind(subject)
    .fetch_optional(&state.pool)
    .await
    .map_err(errors::server_error)?;
    if matches!(existing.as_deref(), Some("pending" | "done")) {
        return done_with(
            &state,
            &viewer,
            None,
            "info",
            "This standing has already been requested.",
        )
        .await;
    }
    let now = state.clock.now();
    let request_id: i64 = sqlx::query_scalar(
        "insert into standing_requests (subject_type, subject_id, requested_by_character_id, status,
            created_at, updated_at)
         values ($1, $2, $3, 'pending', $4, $4)
         on conflict (subject_type, subject_id) do update set
            requested_by_character_id = excluded.requested_by_character_id,
            status = 'pending', updated_at = excluded.updated_at
         returning id",
    )
    .bind(&kind)
    .bind(subject)
    .bind(character.id)
    .bind(now)
    .fetch_one(&state.pool)
    .await
    .map_err(errors::server_error)?;
    jobs::enqueue(
        &state,
        &Job::DiscordRequest {
            standing_request_id: request_id,
        },
    )
    .await
    .map_err(errors::server_error)?;
    done_with(
        &state,
        &viewer,
        None,
        "success",
        "Standing request submitted.",
    )
    .await
}

/// `PUT /auth/character/{character}`: the active character.
pub async fn switch(
    State(state): State<AppState>,
    viewer: Viewer,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    let account = account::form_account(&state, &viewer).await?;
    let character = account::owned_character(&state, &account, &id).await?;
    account::set_active(&state, &account.session_id, character.id)
        .await
        .map_err(errors::server_error)?;
    done_with(
        &state,
        &viewer,
        None,
        "success",
        &format!("Switched to {}.", display(&character.name)),
    )
    .await
}

/// `PUT /auth/character/{character}/main`, which also finishes the
/// setup.
pub async fn set_main(
    State(state): State<AppState>,
    viewer: Viewer,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    let account = account::form_account(&state, &viewer).await?;
    let character = account::owned_character(&state, &account, &id).await?;
    sqlx::query(
        "update users set main_character_id = $2, setup_completed_at = coalesce(setup_completed_at, $3),
            updated_at = $3
         where id = $1",
    )
    .bind(account.id)
    .bind(character.id)
    .bind(state.clock.now())
    .execute(&state.pool)
    .await
    .map_err(errors::server_error)?;
    done_with(
        &state,
        &viewer,
        None,
        "success",
        &format!("{} is now your main character.", display(&character.name)),
    )
    .await
}

/// `RemoveCharacterFromUserAction`: the character leaves the account, its
/// tokens go, and it stops syncing (FIXES.md, "Removed characters were
/// mailed about lapsed tokens forever").
async fn remove(state: &AppState, account: &Account, character_id: i64) -> sqlx::Result<()> {
    let now = state.clock.now();
    let mut tx = state.pool.begin().await?;
    sqlx::query("update users set main_character_id = null, updated_at = $2 where id = $1 and main_character_id = $3")
        .bind(account.id)
        .bind(now)
        .bind(character_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("delete from esi_tokens where character_id = $1")
        .bind(character_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "update characters set user_id = null, character_owner_hash = null, should_sync = false,
            updated_at = $2
         where id = $1",
    )
    .bind(character_id)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

/// `DELETE /auth/character/{character}`: removes a character; the last
/// one takes the account with it and signs out.
pub async fn destroy(
    State(state): State<AppState>,
    viewer: Viewer,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    let account = account::form_account(&state, &viewer).await?;
    let character = account::owned_character(&state, &account, &id).await?;
    let last = account.characters.len() <= 1;
    remove(&state, &account, character.id)
        .await
        .map_err(errors::server_error)?;
    if last {
        session::end(&state.pool, &account.session_id)
            .await
            .map_err(errors::server_error)?;
        sqlx::query("delete from users where id = $1")
            .bind(account.id)
            .execute(&state.pool)
            .await
            .map_err(errors::server_error)?;
        // A new guest session carries the message to the home page.
        let mut response = Done::to("/").into_response();
        viewer::flash(
            &state,
            &Viewer::default(),
            "success",
            "Your account has been deleted.",
            &mut response,
        )
        .await
        .map_err(errors::server_error)?;
        return Ok(response);
    }
    let first: Option<i64> =
        sqlx::query_scalar("select id from characters where user_id = $1 order by id limit 1")
            .bind(account.id)
            .fetch_optional(&state.pool)
            .await
            .map_err(errors::server_error)?;
    if let Some(first) = first {
        account::set_active(&state, &account.session_id, first)
            .await
            .map_err(errors::server_error)?;
    }
    done_with(
        &state,
        &viewer,
        None,
        "success",
        &format!(
            "{} has been removed from your account.",
            display(&character.name)
        ),
    )
    .await
}

/// `POST /setup/complete`: the wizard was finished or skipped.
pub async fn complete_setup(
    State(state): State<AppState>,
    viewer: Viewer,
) -> Result<Response, Response> {
    let account = account::form_account(&state, &viewer).await?;
    sqlx::query(
        "update users set setup_completed_at = $2, updated_at = $2 where id = $1 and setup_completed_at is null",
    )
    .bind(account.id)
    .bind(state.clock.now())
    .execute(&state.pool)
    .await
    .map_err(errors::server_error)?;
    Ok(Done::back().into_response())
}

/// `/settings`, every method: Laravel's redirect to the appearance page,
/// behind the login.
pub async fn settings_redirect(
    State(state): State<AppState>,
    viewer: Viewer,
    uri: Uri,
) -> Result<Response, Response> {
    let page = uri
        .path_and_query()
        .map_or("/settings", |path| path.as_str());
    account::page_account(&state, &viewer, page).await?;
    Ok(answer::redirect("/settings/appearance"))
}

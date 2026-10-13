//! The Discord messages (new standing requests, an unreadable source): the
//! webhook of the admin settings, else DISCORD_STANDING_REQUEST_WEBHOOK;
//! the role of the settings is pinged. The answer is not looked at.

use serde_json::{Value, json};

use crate::scopes::{ADMIN_SCOPES, grant_url};
use crate::server::AppState;
use crate::standings::Source;

/// The webhook and the role to ping.
async fn settings(state: &AppState) -> sqlx::Result<(Option<String>, Option<String>)> {
    let row: Option<(Option<String>, Option<String>)> =
        sqlx::query_as("select webhook_url, role_id from discord_settings order by id limit 1")
            .fetch_optional(&state.pool)
            .await?;
    let (webhook, role) = row.unwrap_or((None, None));
    let webhook = webhook
        .filter(|url| !url.is_empty())
        .or_else(|| state.config.discord_webhook.clone());
    Ok((webhook, role.filter(|role| !role.is_empty())))
}

async fn post(state: &AppState, webhook: &str, payload: &Value) {
    let client = reqwest::Client::new();
    match client
        .post(webhook)
        .header("user-agent", state.config.esi.user_agent.as_str())
        .json(payload)
        .send()
        .await
    {
        Ok(response) if !response.status().is_success() => {
            tracing::warn!(status = %response.status(), "Discord refused the message");
        }
        Ok(_) => {}
        Err(error) => tracing::warn!(%error, "Discord unreachable"),
    }
}

fn with_mentions(mut payload: Value, role: &Option<String>) -> Value {
    if let Some(role) = role {
        payload["allowed_mentions"] = json!({ "roles": [role] });
    }
    payload
}

fn ping(role: &Option<String>) -> String {
    role.as_ref()
        .map(|role| format!("<@&{role}> "))
        .unwrap_or_default()
}

/// The image server's portrait or logo of an entity.
pub fn image_url(kind: &str, id: i64) -> String {
    match kind {
        "character" => format!("https://images.evetech.net/characters/{id}/portrait?size=128"),
        "corporation" => format!("https://images.evetech.net/corporations/{id}/logo?size=128"),
        _ => format!("https://images.evetech.net/alliances/{id}/logo?size=128"),
    }
}

fn label(kind: &str) -> String {
    let mut chars = kind.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

/// A request's subject and its requester's names.
#[derive(sqlx::FromRow)]
struct RequestNames {
    kind: String,
    subject_id: i64,
    subject: Option<String>,
    requester: Option<String>,
}

/// Announces a standing request; an error when the request is gone.
pub async fn notify_request(state: &AppState, standing_request_id: i64) -> Result<(), String> {
    let request: Option<RequestNames> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select r.subject_type as kind, r.subject_id, {} as subject, c.name as requester
             from standing_requests r
             join characters c on c.id = r.requested_by_character_id
             where r.id = $1",
        crate::standings::SUBJECT_NAME_SQL
    )))
    .bind(standing_request_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| error.to_string())?;
    let Some(RequestNames {
        kind,
        subject_id,
        subject,
        requester,
    }) = request
    else {
        return Err(format!(
            "standing request {standing_request_id} does not exist"
        ));
    };
    let (webhook, role) = settings(state).await.map_err(|error| error.to_string())?;
    let Some(webhook) = webhook else {
        return Ok(());
    };
    let subject = subject.unwrap_or_else(|| subject_id.to_string());
    let requester = requester.unwrap_or_default();
    let payload = json!({
        "content": format!("{}A {kind} requested standings.", ping(&role)),
        "embeds": [{
            "title": subject,
            // The name in a code block so it can be copied with one click.
            "description": format!(
                "Requested by {requester}. Review it in the administration page.\n```\n{subject}\n```"
            ),
            "url": state.config.url("/admin"),
            "thumbnail": { "url": image_url(&kind, subject_id) },
            "fields": [
                { "name": "Type", "value": label(&kind), "inline": true },
                { "name": "Requested by", "value": requester, "inline": true },
            ],
        }],
    });
    post(state, &webhook, &with_mentions(payload, &role)).await;
    Ok(())
}

/// Alerts that the source cannot be read, with the admin grant link so
/// an admin can grant the missing scopes in one click.
pub async fn notify_unreadable(state: &AppState) -> Result<(), String> {
    let (webhook, role) = settings(state).await.map_err(|error| error.to_string())?;
    let Some(webhook) = webhook else {
        return Ok(());
    };
    let source = Source::current(&state.pool)
        .await
        .map_err(|error| error.to_string())?;
    let name = match &source {
        Some(source) => source
            .entity_name(&state.pool)
            .await
            .map_err(|error| error.to_string())?
            .unwrap_or_else(|| source.entity_id.to_string()),
        None => "the source".to_owned(),
    };
    let login = grant_url(&state.config.app_url, &ADMIN_SCOPES);
    let payload = json!({
        "content": format!("{}:warning: The standings source could not be read.", ping(&role)),
        "embeds": [{
            "title": "Standings source unreadable",
            "description": format!(
                "Couldn't read **{name}**'s contacts. An admin character's ESI token may be missing or lacking permissions.\n\n**[Click here to re-authenticate]({login})**"
            ),
            "url": login,
        }],
    });
    post(state, &webhook, &with_mentions(payload, &role)).await;
    Ok(())
}

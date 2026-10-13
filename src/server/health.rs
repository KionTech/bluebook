//! Health checks: `/up` (a page, or `{"status":"up"}` for a JSON client)
//! and the deploy check `/api/health`.

use std::time::Instant;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use serde_json::json;

use super::AppState;
use super::errors::{escape, expects_json};

/// The health page.
const UP_PAGE: &str = include_str!("up.html");

pub async fn up(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let started = Instant::now();
    if expects_json(&headers) {
        return Json(json!({ "status": "up" })).into_response();
    }
    Html(
        UP_PAGE
            .replace("{{app_name}}", &escape(&state.config.app_name))
            .replace(
                "{{milliseconds}}",
                &started.elapsed().as_millis().to_string(),
            ),
    )
    .into_response()
}

/// Up, and the database answers.
pub async fn health(State(state): State<AppState>) -> Response {
    match sqlx::query("select 1").execute(&state.pool).await {
        Ok(_) => Json(json!({ "status": "ok", "database": "ok" })).into_response(),
        Err(error) => {
            tracing::error!(%error, "health check: database unreachable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "error", "database": "unreachable" })),
            )
                .into_response()
        }
    }
}

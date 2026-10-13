//! Who is asking: the session cookie resolved once per request, before
//! any handler runs, plus the helpers that write to the session (the flash message,
//! the intended URL), starting a guest session when there is none.

use axum::extract::{FromRequestParts, Request, State};
use axum::http::HeaderValue;
use axum::http::header::SET_COOKIE;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use serde_json::{Map, Value, json};

use super::AppState;
use crate::auth::session::{self, SESSION_COOKIE, Session};

/// The request's session, if it has one.
#[derive(Debug, Clone, Default)]
pub struct Viewer {
    pub session: Option<Session>,
}

impl Viewer {
    /// The signed-in user's id.
    pub fn user_id(&self) -> Option<i64> {
        self.session.as_ref().and_then(|session| session.user_id)
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session.as_ref().map(|session| session.id.as_str())
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.session
            .as_ref()
            .and_then(|session| session.get_i64(key))
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Viewer {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<Viewer>()
            .cloned()
            .unwrap_or_default())
    }
}

/// Resolves the viewer of every request.
pub async fn resolve(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let mut session = None;
    if let Some(token) = session::cookie_value(request.headers(), SESSION_COOKIE) {
        session = session::resolve(
            &state.pool,
            &token,
            state.clock.now(),
            state.config.session_lifetime_days,
        )
        .await
        .unwrap_or_else(|error| {
            tracing::error!(%error, "session lookup failed");
            None
        });
    }
    request.extensions_mut().insert(Viewer { session });
    next.run(request).await
}

/// Puts a value into the viewer's session, starting a guest session (and
/// adding its cookie to the answer) when there is none.
pub async fn put(
    state: &AppState,
    viewer: &Viewer,
    key: &str,
    value: Value,
    response: &mut Response,
) -> sqlx::Result<()> {
    match viewer.session_id() {
        Some(id) => session::put(&state.pool, id, key, value).await,
        None => {
            let mut data = Map::new();
            data.insert(key.to_owned(), value);
            let token = session::start(&state.pool, None, data, state.clock.now()).await?;
            let cookie = session::cookie(
                token,
                state.config.session_lifetime_days,
                state.config.secure_cookies(),
            );
            if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
                response.headers_mut().append(SET_COOKIE, value);
            }
            Ok(())
        }
    }
}

/// The flash message of the next page: `success`, `error` or `info`.
pub fn flash_value(kind: &str, message: &str) -> Value {
    json!({ "kind": kind, "message": message })
}

/// Flashes a message into the viewer's session.
pub async fn flash(
    state: &AppState,
    viewer: &Viewer,
    kind: &str,
    message: &str,
    response: &mut Response,
) -> sqlx::Result<()> {
    put(state, viewer, "flash", flash_value(kind, message), response).await
}

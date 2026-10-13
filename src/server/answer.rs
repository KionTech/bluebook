//! The answers of the forms. Laravel redirected back and flashed the
//! message into the session; the pages send fetch requests and get the
//! outcome as JSON (FIXES.md, "Forms answer JSON"):
//!
//! - `200 {"redirect": path | null}`: where to go next (null: stay and
//!   reload the page, Laravel's `back()`); the flash message waits in the
//!   session for the next page's props;
//! - `422 {"errors": {field: first message}}`, Inertia's errors bag;
//! - `401 {"message", "redirect": "/eve"}` for a guest, `403`, `404`,
//!   `419` (another site) with `{"message"}`.

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::Cookie;
use serde_json::{Map, Value, json};

/// A form that worked.
pub struct Done {
    pub redirect: Option<String>,
    pub cookies: Vec<Cookie<'static>>,
}

impl Done {
    /// Stay on the page and reload it (Laravel's `back()`).
    pub fn back() -> Self {
        Self {
            redirect: None,
            cookies: Vec::new(),
        }
    }

    pub fn to(path: impl Into<String>) -> Self {
        Self {
            redirect: Some(path.into()),
            cookies: Vec::new(),
        }
    }

    pub fn cookie(mut self, cookie: Cookie<'static>) -> Self {
        self.cookies.push(cookie);
        self
    }
}

impl IntoResponse for Done {
    fn into_response(self) -> Response {
        let mut response = Json(json!({ "redirect": self.redirect })).into_response();
        for cookie in self.cookies {
            if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
                response.headers_mut().append(header::SET_COOKIE, value);
            }
        }
        response
    }
}

/// Each field's first validation message, in the order of the rules.
#[derive(Debug, Default, Clone)]
pub struct Errors(Map<String, Value>);

impl Errors {
    /// Keeps the first message of a field.
    pub fn add(&mut self, field: &str, message: impl Into<String>) {
        if !self.0.contains_key(field) {
            self.0
                .insert(field.to_owned(), Value::String(message.into()));
        }
    }

    pub fn has(&self, field: &str) -> bool {
        self.0.contains_key(field)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// `Ok` without errors, the 422 answer with them.
    pub fn finish(self) -> Result<(), Response> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(invalid(self))
        }
    }
}

/// A form with errors.
pub fn invalid(errors: Errors) -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({ "errors": Value::Object(errors.0) })),
    )
        .into_response()
}

/// One field's error.
pub fn invalid_field(field: &str, message: &str) -> Response {
    let mut errors = Errors::default();
    errors.add(field, message);
    invalid(errors)
}

/// A guest's form: the login, as Laravel's `auth` middleware redirected
/// there.
pub fn unauthenticated() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "message": "Unauthenticated.", "redirect": "/eve" })),
    )
        .into_response()
}

/// A plain redirect (the login, Laravel's `Route::redirect`).
pub fn redirect(location: &str) -> Response {
    (StatusCode::FOUND, [(header::LOCATION, location.to_owned())]).into_response()
}

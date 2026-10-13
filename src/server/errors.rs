//! Error answers: `{"message": ...}` for the api paths and for clients
//! that ask for JSON (the pages' fetches), the minimal error page
//! otherwise.

use axum::Json;
use axum::extract::Request;
use axum::http::{HeaderMap, Method, StatusCode, Uri, header};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Response};
use serde_json::json;

/// The minimal error page with the status and its message.
const ERROR_PAGE: &str = include_str!("error.html");

/// The title of a status, as the error pages show it.
pub fn title(status: StatusCode) -> &'static str {
    match status.as_u16() {
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        419 => "Page Expired",
        429 => "Too Many Requests",
        503 => "Service Unavailable",
        _ => "Server Error",
    }
}

/// `{"message": ...}` with the status, always JSON.
pub fn json_error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "message": message }))).into_response()
}

/// The error page of a status. Only 403 shows its own message; the others
/// their title.
pub fn error_page(status: StatusCode, message: &str) -> Response {
    let shown = if status == StatusCode::FORBIDDEN && !message.is_empty() {
        message
    } else {
        title(status)
    };
    let page = ERROR_PAGE
        .replace("{{title}}", title(status))
        .replace("{{status}}", status.as_str())
        .replace("{{message}}", &escape(shown));
    (status, Html(page)).into_response()
}

/// HTML escaping for text in markup.
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#039;")
}

/// Marks an error answer that becomes the error page for a browser (see
/// `negotiate`), with the message the page shows.
#[derive(Clone)]
struct PageForBrowsers(String);

/// An error from the exception handler: `{"message": ...}` for JSON
/// clients and the api paths, the error page for the others.
pub fn http_error(status: StatusCode, message: &str) -> Response {
    let mut response = json_error(status, message);
    response
        .extensions_mut()
        .insert(PageForBrowsers(message.to_owned()));
    response
}

/// A route parameter that names no row.
pub fn model_not_found(model: &str, id: &str) -> Response {
    http_error(
        StatusCode::NOT_FOUND,
        &format!("No query results for model [App\\Models\\{model}] {id}"),
    )
}

/// A policy or gate said no.
pub fn forbidden() -> Response {
    http_error(StatusCode::FORBIDDEN, "This action is unauthorized.")
}

/// Any failure the client cannot fix: logged, answered with the generic
/// message.
pub fn server_error(error: impl std::fmt::Display) -> Response {
    tracing::error!(%error, "request failed");
    http_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        title(StatusCode::INTERNAL_SERVER_ERROR),
    )
}

/// The path as messages name it: without the leading slash, `/` for the
/// root.
fn route_path(uri: &Uri) -> String {
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() {
        "/".to_owned()
    } else {
        path.to_owned()
    }
}

/// Unknown paths.
pub async fn not_found(uri: Uri) -> Response {
    http_error(
        StatusCode::NOT_FOUND,
        &format!("The route {} could not be found.", route_path(&uri)),
    )
}

/// Whether the client asked for JSON: the most wanted type of its
/// `Accept` header is JSON, or it is an XHR that takes anything.
pub fn expects_json(headers: &HeaderMap) -> bool {
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let mut types: Vec<(f32, usize, String)> = accept
        .split(',')
        .enumerate()
        .filter_map(|(index, part)| {
            let mut pieces = part.split(';');
            let media = pieces.next()?.trim().to_ascii_lowercase();
            if media.is_empty() {
                return None;
            }
            let quality = pieces
                .filter_map(|parameter| parameter.trim().strip_prefix("q="))
                .find_map(|q| q.parse::<f32>().ok())
                .unwrap_or(1.0);
            Some((quality, index, media))
        })
        .collect();
    types.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let first = types.first().map(|(_, _, media)| media.as_str());
    let wants_json = first.is_some_and(|media| media.contains("/json") || media.contains("+json"));
    let xhr = headers
        .get("x-requested-with")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == "XMLHttpRequest");
    let accepts_anything = matches!(first, None | Some("*/*") | Some("*"));
    wants_json || (xhr && accepts_anything)
}

/// Renders the error answers for the client: the error page unless it
/// asked for JSON or called the api, and the supported methods of a 405.
pub async fn negotiate(request: Request, next: Next) -> Response {
    let wants_json = expects_json(request.headers()) || request.uri().path().starts_with("/api/");
    let method = request.method().clone();
    let uri = request.uri().clone();
    let mut response = next.run(request).await;
    if response.status() == StatusCode::METHOD_NOT_ALLOWED {
        response = method_not_allowed_answer(&method, &uri, response);
    }
    if let Some(PageForBrowsers(message)) = response.extensions().get::<PageForBrowsers>().cloned()
        && !wants_json
    {
        let headers = response.headers().clone();
        let mut page = error_page(response.status(), &message);
        for name in [header::ALLOW, header::SET_COOKIE] {
            for value in headers.get_all(&name) {
                page.headers_mut().append(&name, value.clone());
            }
        }
        return page;
    }
    response
}

fn method_not_allowed_answer(method: &Method, uri: &Uri, response: Response) -> Response {
    let methods: Vec<String> = response
        .headers()
        .get(header::ALLOW)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .split(',')
        .map(|method| method.trim().to_owned())
        .filter(|method| !method.is_empty())
        .collect();
    let allow = methods.join(", ");
    let mut answer = http_error(
        StatusCode::METHOD_NOT_ALLOWED,
        &format!(
            "The {method} method is not supported for route {}. Supported methods: {allow}.",
            route_path(uri)
        ),
    );
    if let Ok(value) = allow.parse() {
        answer.headers_mut().insert(header::ALLOW, value);
    }
    answer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accept(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, value.parse().expect("a header"));
        headers
    }

    #[test]
    fn reads_the_most_wanted_type() {
        assert!(expects_json(&accept("application/json")));
        assert!(expects_json(&accept("text/html;q=0.5, application/json")));
        assert!(!expects_json(&accept("text/html, application/json")));
        assert!(!expects_json(&accept("*/*")));
        assert!(!expects_json(&HeaderMap::new()));
    }

    #[test]
    fn the_forbidden_page_shows_its_message() {
        let page = error_page(
            StatusCode::FORBIDDEN,
            "No characters found. Please log in again.",
        );
        assert_eq!(page.status(), StatusCode::FORBIDDEN);
    }
}

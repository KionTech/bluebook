//! The HTTP side: shared state, the router and the cross-cutting layers.
//! Pages are rendered by SvelteKit; the api serves their props as JSON
//! (`/api/pages/*`), the EVE login, and every form the pages send, at the
//! URL and method the site always used.

pub mod errors;
pub mod health;
pub mod viewer;

use std::sync::Arc;

use axum::Router;
use axum::extract::Request;
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post, put};
use sqlx::PgPool;
use tokio::sync::Notify;
use tower_http::normalize_path::NormalizePath;

use crate::clock::Clock;
use crate::config::Config;

/// The default bind address of the API server; the SvelteKit dev proxy and
/// the reverse proxy point at it.
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";

/// The listen address from `BIND_ADDR`, with the local default.
pub fn bind_addr() -> String {
    std::env::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned())
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub clock: Clock,
    /// Wakes the queue workers when a job is queued.
    pub jobs: Arc<Notify>,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config, clock: Clock) -> Self {
        Self {
            pool,
            config: Arc::new(config),
            clock,
            jobs: Arc::new(Notify::new()),
        }
    }
}

/// The whole app. A trailing slash reaches the same route, as it did.
/// Error answers are rendered for the client around the routing, so they
/// see every answer as the router finished it (the `Allow` header of a
/// 405 included).
pub fn router(state: AppState) -> Router {
    let routes = routes(state);
    let app = tower::ServiceBuilder::new()
        .layer(axum::middleware::from_fn(errors::negotiate))
        .service(NormalizePath::trim_trailing_slash(routes));
    Router::new().fallback_service(app)
}

fn routes(state: AppState) -> Router {
    use errors::not_implemented;
    Router::new()
        .route("/up", get(health::up))
        // The EVE login.
        .route("/eve", get(not_implemented))
        .route("/eve/callback", get(not_implemented))
        .route("/logout", post(not_implemented))
        // The pilot's forms.
        .route("/standings/sync", post(not_implemented))
        .route("/standings/{character}", put(not_implemented))
        .route("/standings/{character}/request", post(not_implemented))
        .route(
            "/auth/character/{character}",
            put(not_implemented).delete(not_implemented),
        )
        .route("/auth/character/{character}/main", put(not_implemented))
        .route("/setup/complete", post(not_implemented))
        .route("/settings", any(not_implemented))
        // The admin's forms.
        .route("/admin/standings", put(not_implemented))
        .route("/admin/standings/sync", post(not_implemented))
        .route(
            "/admin/standing-requests/{standing_request}",
            put(not_implemented),
        )
        .route("/admin/discord-settings", put(not_implemented))
        // The local disk's routes; nothing is stored there.
        .route(
            "/storage/{*path}",
            get(not_implemented).put(not_implemented),
        )
        // The pages are SvelteKit's; any other method is not allowed.
        .route("/", get(page_elsewhere))
        .route("/dashboard", get(page_elsewhere))
        .route("/settings/appearance", get(page_elsewhere))
        .route("/admin", get(page_elsewhere))
        .route("/admin/standing-requests", get(page_elsewhere))
        .route("/admin/pilots", get(page_elsewhere))
        .route("/admin/settings", get(page_elsewhere))
        .nest("/api", api_router())
        .fallback(errors::not_found)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            viewer::resolve,
        ))
        .layer(axum::middleware::from_fn(reject_cross_site_mutations))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            reload_inertia_requests,
        ))
        .with_state(state)
}

/// One props endpoint per page, at the page's own path under
/// `/api/pages` (the home page is `/api/pages/home`), and the health
/// check.
fn api_router() -> Router<AppState> {
    use errors::not_implemented;
    Router::new()
        .route("/health", get(health::health))
        .route("/pages/home", get(not_implemented))
        .route("/pages/dashboard", get(not_implemented))
        .route("/pages/settings/appearance", get(not_implemented))
        .route("/pages/admin", get(not_implemented))
        .route("/pages/admin/standing-requests", get(not_implemented))
        .route("/pages/admin/pilots", get(not_implemented))
        .route("/pages/admin/settings", get(not_implemented))
}

/// A page path reached the api: SvelteKit serves it, the proxy never sends
/// a GET here.
async fn page_elsewhere(uri: axum::http::Uri) -> Response {
    errors::not_found(uri).await
}

/// Tabs left open on the Laravel site still send Inertia's XHR
/// (`X-Inertia: true`). A 409 with `X-Inertia-Location` makes Inertia load
/// that URL in full: the same page for a GET, the page a form was on for
/// anything else (its Referer, or the home page).
async fn reload_inertia_requests(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if !request.headers().contains_key("x-inertia") {
        return next.run(request).await;
    }
    let headers = request.headers();
    let location = if matches!(*request.method(), Method::GET | Method::HEAD) {
        let scheme = headers
            .get("x-forwarded-proto")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_else(|| state.config.app_url.split("://").next().unwrap_or("https"));
        let host = headers
            .get("x-forwarded-host")
            .or_else(|| headers.get(header::HOST))
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        let path = request
            .uri()
            .path_and_query()
            .map_or("/", |path| path.as_str());
        format!("{scheme}://{host}{path}")
    } else {
        headers
            .get(header::REFERER)
            .and_then(|value| value.to_str().ok())
            .filter(|referer| !is_cross_site_url(referer, headers))
            .map_or_else(|| format!("{}/", state.config.app_url), str::to_owned)
    };
    let mut response = StatusCode::CONFLICT.into_response();
    if let Ok(location) = location.parse() {
        response
            .headers_mut()
            .insert("x-inertia-location", location);
    }
    response
}

/// The same-origin check that replaces Laravel's CSRF token: a mutation
/// the browser marks as coming from another site, or whose `Origin` is not
/// this host, is refused with Laravel's 419. Requests without either
/// header (non-browser clients) cannot be forged by a third-party page and
/// pass. The session cookie is `SameSite=Lax` on top.
async fn reject_cross_site_mutations(request: Request, next: Next) -> Response {
    let read_only = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    if read_only {
        return next.run(request).await;
    }
    let headers = request.headers();
    let fetch_site_foreign = headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|site| site != "same-origin" && site != "none");
    let origin_foreign = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|origin| is_cross_site_url(origin, headers));
    if fetch_site_foreign || origin_foreign {
        return errors::http_error(
            StatusCode::from_u16(419).expect("419 is a status"),
            "CSRF token mismatch.",
        );
    }
    next.run(request).await
}

/// Whether a URL (an `Origin` or `Referer`) names a host other than the
/// one serving the request (the host the proxy forwarded, or `Host`); an
/// opaque `null` origin counts as foreign.
fn is_cross_site_url(url: &str, headers: &HeaderMap) -> bool {
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let authority = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .and_then(|rest| rest.split('/').next())
        .unwrap_or_default();
    authority.is_empty() || !authority.eq_ignore_ascii_case(host)
}

//! Runtime configuration from the environment, resolved once at startup.
//! Tests build a `Config` by hand instead of mutating process env vars.

use std::time::Duration;

/// Everything the app reads from the environment.
#[derive(Debug, Clone)]
pub struct Config {
    /// `APP_NAME`: the shared `name` prop and the page title suffix.
    pub app_name: String,
    /// `APP_URL` without a trailing slash: the grant links, the Discord
    /// links and the redirects after a login.
    pub app_url: String,
    pub eve: EveConfig,
    pub esi: EsiConfig,
    /// `EVE_ADMIN_CHARACTER_ID`: comma separated character ids; whoever
    /// owns one of them administers the standings.
    pub admin_character_ids: Vec<i64>,
    /// `DISCORD_STANDING_REQUEST_WEBHOOK`: the webhook when the admin
    /// settings name none.
    pub discord_webhook: Option<String>,
    /// `SCHEDULER_ENABLED`: runs the standings sync, the affiliation
    /// refresh and the token check on their schedule. Off unless asked
    /// for, so a local stack or a staging copy never touches the game.
    pub scheduler_enabled: bool,
    /// `QUEUE_WORKERS`: how many jobs run at once.
    pub queue_workers: usize,
    /// `SESSION_LIFETIME_DAYS`: how long a sign-in lasts.
    pub session_lifetime_days: i64,
    /// `DEMO_ENABLED` and `DEMO_CHARACTER_ID`: the one-click demo login
    /// for local stacks, off unless asked for.
    pub demo: DemoConfig,
}

/// The EVE SSO application.
#[derive(Debug, Clone)]
pub struct EveConfig {
    /// `SSO_BASE_URL`.
    pub sso_base_url: String,
    /// `EVE_CLIENT_ID` and `EVE_CLIENT_SECRET` (or the Laravel names
    /// `EVEONLINE_CLIENT_ID` and `EVEONLINE_CLIENT_SECRET`).
    pub client_id: String,
    pub client_secret: String,
    /// `EVE_CALLBACK_URL` (or `EVEONLINE_REDIRECT_URI`): the callback
    /// registered with CCP.
    pub callback_url: String,
    /// The pause between tries of a token refresh that EVE answered with
    /// a server error.
    pub refresh_retry_delay: Duration,
}

/// EVE's API.
#[derive(Debug, Clone)]
pub struct EsiConfig {
    /// `ESI_BASE_URL`.
    pub base_url: String,
    /// `ESI_USER_AGENT`.
    pub user_agent: String,
    /// `ESI_MAX_RPS`: the cap on requests per second, 0 for none.
    pub max_rps: f64,
    /// The pause between tries of a read that ESI answered with a server
    /// error.
    pub retry_delay: Duration,
}

#[derive(Debug, Clone)]
pub struct DemoConfig {
    pub enabled: bool,
    pub character_id: i64,
}

pub const DEFAULT_APP_NAME: &str = "Bluebook";
pub const DEFAULT_APP_URL: &str = "https://bluebook.nicolaskion.dev";
pub const DEFAULT_SSO_BASE_URL: &str = "https://login.eveonline.com";
pub const DEFAULT_CALLBACK_URL: &str = "https://bluebook.nicolaskion.dev/eve/callback";
pub const DEFAULT_ESI_BASE_URL: &str = "https://esi.evetech.net";
pub const DEFAULT_ESI_USER_AGENT: &str = "Bluebook (+https://bluebook.nicolaskion.dev)";
pub const DEFAULT_ESI_MAX_RPS: f64 = 20.0;
pub const DEFAULT_QUEUE_WORKERS: usize = 5;
pub const DEFAULT_SESSION_LIFETIME_DAYS: i64 = 400;
pub const DEFAULT_DEMO_CHARACTER_ID: i64 = 2_100_000_001;
/// Five seconds between tries of an ESI read, one between tries of a
/// token refresh.
pub const ESI_RETRY_DELAY: Duration = Duration::from_secs(5);
pub const REFRESH_RETRY_DELAY: Duration = Duration::from_secs(1);

/// A variable that is set and not empty (`null`, as Laravel's .env files
/// write it, counts as unset).
fn env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .filter(|value| !value.is_empty() && value != "null")
}

fn env_or(key: &str, default: &str) -> String {
    env(key).unwrap_or_else(|| default.to_owned())
}

/// A switch: `true`, `1`, `on` or `yes`.
fn env_flag(key: &str) -> bool {
    env(key).is_some_and(|value| {
        matches!(
            value.to_ascii_lowercase().as_str(),
            "true" | "1" | "on" | "yes"
        )
    })
}

/// A number from the environment; one that does not parse stops the
/// startup instead of running with a surprise.
fn env_number<T: std::str::FromStr>(key: &str, default: T) -> T {
    env(key).map_or(default, |value| {
        value
            .parse()
            .unwrap_or_else(|_| panic!("{key} is {value:?}, expected a number"))
    })
}

/// `EVE_ADMIN_CHARACTER_ID`'s list: the ids between the commas, trimmed;
/// anything that is not a number is left out, as `(int)` and
/// `array_filter` left it out.
pub fn parse_admin_ids(value: &str) -> Vec<i64> {
    value
        .split(',')
        .filter_map(|id| id.trim().parse::<i64>().ok())
        .filter(|id| *id != 0)
        .collect()
}

impl Config {
    pub fn from_env() -> Self {
        let app_url = env_or("APP_URL", DEFAULT_APP_URL)
            .trim_end_matches('/')
            .to_owned();
        Self {
            app_name: env_or("APP_NAME", DEFAULT_APP_NAME),
            eve: EveConfig {
                sso_base_url: env_or("SSO_BASE_URL", DEFAULT_SSO_BASE_URL)
                    .trim_end_matches('/')
                    .to_owned(),
                client_id: env("EVE_CLIENT_ID")
                    .or_else(|| env("EVEONLINE_CLIENT_ID"))
                    .unwrap_or_default(),
                client_secret: env("EVE_CLIENT_SECRET")
                    .or_else(|| env("EVEONLINE_CLIENT_SECRET"))
                    .unwrap_or_default(),
                callback_url: env("EVE_CALLBACK_URL")
                    .or_else(|| env("EVEONLINE_REDIRECT_URI"))
                    .unwrap_or_else(|| DEFAULT_CALLBACK_URL.to_owned()),
                refresh_retry_delay: REFRESH_RETRY_DELAY,
            },
            esi: EsiConfig {
                base_url: env_or("ESI_BASE_URL", DEFAULT_ESI_BASE_URL)
                    .trim_end_matches('/')
                    .to_owned(),
                user_agent: env_or("ESI_USER_AGENT", DEFAULT_ESI_USER_AGENT),
                max_rps: env_number("ESI_MAX_RPS", DEFAULT_ESI_MAX_RPS),
                retry_delay: ESI_RETRY_DELAY,
            },
            admin_character_ids: parse_admin_ids(&env_or("EVE_ADMIN_CHARACTER_ID", "")),
            discord_webhook: env("DISCORD_STANDING_REQUEST_WEBHOOK"),
            scheduler_enabled: env_flag("SCHEDULER_ENABLED"),
            queue_workers: env_number("QUEUE_WORKERS", DEFAULT_QUEUE_WORKERS).max(1),
            session_lifetime_days: env_number(
                "SESSION_LIFETIME_DAYS",
                DEFAULT_SESSION_LIFETIME_DAYS,
            ),
            demo: DemoConfig {
                enabled: env_flag("DEMO_ENABLED"),
                character_id: env_number("DEMO_CHARACTER_ID", DEFAULT_DEMO_CHARACTER_ID),
            },
            app_url,
        }
    }

    /// Cookies are `Secure` when the site is served over HTTPS.
    pub fn secure_cookies(&self) -> bool {
        self.app_url.starts_with("https://")
    }

    /// An absolute URL on the site.
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.app_url)
    }

    pub fn is_admin_character(&self, character_id: i64) -> bool {
        self.admin_character_ids.contains(&character_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_admin_list_like_the_laravel_config() {
        assert_eq!(parse_admin_ids(""), Vec::<i64>::new());
        assert_eq!(parse_admin_ids("123"), vec![123]);
        assert_eq!(parse_admin_ids(" 1, 2 ,3,"), vec![1, 2, 3]);
        assert_eq!(parse_admin_ids("1,abc,0,4"), vec![1, 4]);
    }
}

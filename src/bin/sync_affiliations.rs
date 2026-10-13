//! `characters:sync-affiliations`, once: refreshes the corporations and
//! alliances of the characters and of the source's contacts.

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    dotenvy::dotenv().ok();
    let config = bluebook::config::Config::from_env();
    let pool = bluebook::db::connect().await.expect("database connection");
    bluebook::db::migrate(&pool)
        .await
        .expect("database migrations");
    let state = bluebook::server::AppState::new(pool, config, bluebook::clock::Clock::system());
    let code = bluebook::entities::sync_affiliations(&state).await;
    std::process::exit(code);
}

#[tokio::main]
async fn main() {
    // Structured logs; tune with RUST_LOG (e.g. RUST_LOG=bluebook=debug).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Local configuration from .env, if present; real environment wins.
    dotenvy::dotenv().ok();

    let config = bluebook::config::Config::from_env();
    let addr = bluebook::server::bind_addr();

    let pool = bluebook::db::connect().await.expect("database connection");
    bluebook::db::migrate(&pool)
        .await
        .expect("database migrations");
    match bluebook::jobs::release_reserved(&pool).await {
        Ok(0) => {}
        Ok(count) => tracing::info!(count, "jobs of the last run released"),
        Err(error) => tracing::error!(%error, "jobs of the last run not released"),
    }

    let state = bluebook::server::AppState::new(pool, config, bluebook::clock::Clock::system());
    bluebook::jobs::spawn_workers(&state);
    bluebook::scheduler::spawn(&state);
    bluebook::scheduler::spawn_housekeeping(&state);
    let app = bluebook::server::router(state);

    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind address");
    axum::serve(listener, app).await.expect("serve");
}

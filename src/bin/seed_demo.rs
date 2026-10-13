//! Seeds the demo pilots (DEMO_CHARACTER_ID and the next ids), a source
//! with its standings and a few requests, for local stacks. Migrates
//! first; safe to repeat.

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = bluebook::config::Config::from_env();
    let pool = bluebook::db::connect().await.expect("database connection");
    bluebook::db::migrate(&pool)
        .await
        .expect("database migrations");
    let now = bluebook::clock::Clock::system().now();
    bluebook::demo::seed(&pool, now, config.demo.character_id)
        .await
        .expect("the demo seed");
    println!(
        "Seeded {} ({}) and the demo standings.",
        bluebook::demo::DEMO_PILOT,
        config.demo.character_id
    );
}

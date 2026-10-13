//! The single integration-test binary: every suite lives here as a module
//! so a full `cargo test` links one executable.
//!
//! The suites assume they never run concurrently with each other: they
//! truncate the shared tables. `.cargo/config.toml` pins
//! `RUST_TEST_THREADS=1` to keep that assumption true.

mod common;

mod access_warnings;
mod admin;
mod admin_pilots;
mod admin_settings;
mod admin_source;
mod affiliations;
mod background;
mod character_sync;
mod characters;
mod check_tokens;
mod dashboard;
mod demo;
mod esi;
mod esi_tokens;
mod forms;
mod harness;
mod jobs;
mod login;
mod login_scopes;
mod page_props;
mod pages;
mod scheduler;
mod sessions;
mod setup;
mod standing_requests;
mod standings_source;
mod standings_sync;
mod visibility;

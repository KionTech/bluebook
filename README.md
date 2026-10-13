# Bluebook

[![CI](https://github.com/KionTech/bluebook/actions/workflows/ci.yml/badge.svg)](https://github.com/KionTech/bluebook/actions/workflows/ci.yml)
[![Website](https://img.shields.io/website?url=https%3A%2F%2Fbluebook.nicolaskion.dev&label=bluebook.nicolaskion.dev)](https://bluebook.nicolaskion.dev)
[![License](https://img.shields.io/badge/license-MIT-blue)](#license)
[![Rust](https://img.shields.io/badge/Rust-Axum-DEA584?logo=rust&logoColor=white)](https://github.com/tokio-rs/axum)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-Svelte%205-FF3E00?logo=svelte&logoColor=white)](https://svelte.dev/docs/kit)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18-4169E1?logo=postgresql&logoColor=white)](https://www.postgresql.org)

Mirrors the contacts of one EVE Online standings source (an admin's
character, corporation or alliance) onto the in-game contact list of every
pilot who opts in, through ESI.

## Features

- Login with EVE SSO; a plain login asks for no scopes, grant links ask
  for the contact scopes (pilots) or the source and mail scopes (admins)
- Several characters per account, a main character, and a first-login
  wizard
- The dashboard: the source, its standings (searchable, with the ones a
  parent corporation or alliance makes redundant marked), the pilot's
  characters with their sync switch, and standing requests
- Standings sync: every five minutes the source is read; when it changed,
  every opted-in character that does not inherit it in game gets its
  contacts written (added, edited, and only previously synced contacts
  removed; personal contacts stay)
- Hourly affiliation refresh, and a daily EVE mail to pilots whose token
  lapsed
- Admin pages: overview, standing requests (approve, reject), pilots by
  character, corporation or alliance with their effective standing, and
  the settings (source, Discord webhook and role)
- Discord messages for new requests and for a source no admin token can
  read
- A one-click demo login with sample data, for local use

## Stack

- API: Rust, [Axum](https://github.com/tokio-rs/axum) and
  [sqlx](https://github.com/launchbadge/sqlx) on Postgres, with the
  schedule and the job workers inside
- Frontend: [SvelteKit](https://svelte.dev/docs/kit) (SSR, adapter-node),
  Svelte 5 and Tailwind CSS 4
- External: EVE SSO, ESI and a Discord webhook

One origin serves both: Caddy (or the Vite dev proxy) sends `/api/*`,
`/up`, `/eve`, `/eve/*`, `/demo-login`, `/settings` and `/storage/*`,
and every request that is not a GET or HEAD, to Axum, and page loads to
SvelteKit. `frontend/proxy-paths.ts` and `deploy/Caddyfile` hold the same
list.

## Getting started

The whole stack (Postgres, API, SvelteKit, Caddy) on
http://localhost:5315:

```bash
docker compose up --build -d
```

The `demo-seed` service seeds the demo pilots, a source with its
standings and a few requests; `/demo-login` signs in as the demo pilot,
who is an admin in this stack. Nothing talks to EVE unless you configure
an EVE application and set `SCHEDULER_ENABLED=true`.

The local preview behind portal
runs on https://bluebook.test:

```bash
APP_URL=https://bluebook.test ORIGIN=https://bluebook.test docker compose up --build -d
```

## Development

```bash
docker compose up -d postgres   # Postgres on 127.0.0.1:5445
cargo run                       # API on 127.0.0.1:3000
cd frontend && npm ci && npm run dev
```

Configuration comes from `.env` (see `.env.example`). The scheduled tasks
also run once as commands: `cargo run --bin sync_standings [-- --force]`,
`sync_affiliations` and `check_tokens`; `seed_demo` seeds the demo and
`migrate` applies the migrations.

## Testing

```bash
cargo test                      # API, against the bluebook_test database
cd frontend
npm run check && npm run lint && npm run format:check
npm test                        # vitest
npx playwright test             # against the running stack on :5315
```

The API suites drive the real router against local mock servers for
ESI, EVE SSO and Discord on ephemeral ports.

## Deployment

Production runs on Coolify from `compose.prod.yml`: `bluebook-api`,
`bluebook-frontend` and `bluebook-proxy`, images built by
`.github/workflows/release.yml` and pushed to
`ghcr.io/kiontech/bluebook-<service>`. Only the proxy gets the domain
https://bluebook.nicolaskion.dev; the api reaches the shared Postgres
over Coolify's network and runs its migrations on startup. The
environment needs `DATABASE_URL`, `EVE_CLIENT_ID`, `EVE_CLIENT_SECRET`,
`EVE_ADMIN_CHARACTER_ID` and optionally `DISCORD_STANDING_REQUEST_WEBHOOK`;
`SCHEDULER_ENABLED=true` switches the schedule on.

## Project layout

```
src/auth/          session, EVE SSO, login, ESI tokens
src/esi.rs         the ESI client
src/standings/     source refresh, character sync, token check, effective standings
src/entities.rs    corporations, alliances, affiliations
src/jobs.rs        the job queue
src/scheduler.rs   the schedule
src/discord.rs     the webhooks
src/server/        router, page props and forms
src/bin/           migrate, seed_demo and the scheduled tasks as commands
migrations/        the Postgres schema
frontend/          the SvelteKit app
deploy/            the Caddyfile and the proxy image
```

## License

MIT, see [LICENSE](LICENSE).

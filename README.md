# Bluebook

[![CI](https://github.com/KionTech/bluebook/actions/workflows/ci.yml/badge.svg)](https://github.com/KionTech/bluebook/actions/workflows/ci.yml)
[![Release](https://github.com/KionTech/bluebook/actions/workflows/release.yml/badge.svg)](https://github.com/KionTech/bluebook/actions/workflows/release.yml)
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

The local preview behind portal runs on https://bluebook.test:

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
docker compose up -d postgres
cargo test
cd frontend && npm run check && npm run lint && npm run format:check && npm test
docker compose up --build -d && cd frontend && npm run test:e2e
```

The Rust suites drive the real router on their own `bluebook_test`
database with a fixed clock, against local mocks of ESI, EVE SSO and
Discord. They pin every page's props, every form with its messages and
changed rows, and every call the scheduled tasks and jobs make. The
vitest suites run next to the code, and Playwright runs against the
composed stack on port 5315, with screenshot baselines of the design
that only run locally.

## Deployment

Coolify deploys `compose.prod.yml`:

1. A merge to main runs `.github/workflows/release.yml`, which builds the
   api, frontend and proxy images and pushes them to
   `ghcr.io/kiontech/bluebook-*` (`latest` and the commit).
2. The workflow then calls Coolify's deploy webhook (`COOLIFY_WEBHOOK`,
   `COOLIFY_TOKEN`), and Coolify pulls the images and restarts the
   services.
3. The api runs its migrations on startup.

Only the proxy has a domain, https://bluebook.nicolaskion.dev; Traefik
terminates TLS in front of it. The api reaches the shared Coolify
Postgres over the `coolify` network. `DATABASE_URL`, `EVE_CLIENT_ID`,
`EVE_CLIENT_SECRET`, `EVE_ADMIN_CHARACTER_ID` and optionally
`DISCORD_STANDING_REQUEST_WEBHOOK` are Coolify environment variables;
`SCHEDULER_ENABLED=true` switches the schedule on.

## Layout

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
tests/integration/ the Rust suites, one binary
deploy/            the Caddyfile and the proxy image
```

## License

MIT, see [LICENSE](LICENSE).

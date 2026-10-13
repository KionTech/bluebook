# bluebook.nicolaskion.dev

Bluebook mirrors the contacts of one EVE Online "standings source" (an
admin's character, corporation or alliance) onto the in-game contact list
of every pilot who opts in, through ESI. Pilots log in with EVE SSO, pick a
main, switch sync per character and can request a standing; admins see the
pilots, resolve requests and pick the source. Discord webhooks announce
requests and an unreadable source; lapsed tokens get an in-game EVE mail.

A Rust API (Axum + Postgres via sqlx) and a SvelteKit frontend (SSR,
adapter-node, Svelte 5, Tailwind 4) in `frontend/`. One shared origin (the
Vite dev proxy locally, Caddy in docker and production) routes backend
paths and every mutation to Axum and page GETs to SvelteKit. Never point
the browser at Axum's port directly. The setup mirrors Medico
(`../Medico`), body-and-face (`../body-and-face`), telescope-web
(`../telescope-web`, EVE SSO) and MutaMarket (`../MutaMarket`, ESI): when
in doubt about structure or tooling, do what they do.

## Details that look arbitrary but are not

- The page props keep their key order (`serde_json` with
  `preserve_order`), and the frontend types in `frontend/src/lib/types.ts`
  follow them key for key. Whole standings have no fraction (`10`, but
  `7.5`), times ISO 8601 with an explicit offset
  (`2026-07-24T12:18:44+00:00`).
- EVE's downtime here is 11:00 to 11:15 UTC (start included, end not).
- The schedule fires on wall clock boundaries (every 5 minutes on :00,
  :05, ...; hourly at :37; daily at 00:00 UTC): the dashboard's countdown
  is `last_synced_at` plus 5 minutes.
- ESI calls carry `X-Compatibility-Date: 2026-07-17`. Reads, affiliations
  and names are tried 5 times, 5 seconds apart, on a 5xx; contact writes
  and mail never. A token expiring within 5 minutes is refreshed first.
  A token is deleted only when the refresh answers `invalid_grant` (or
  `invalid_token`), or ESI answers 401 right after a fresh refresh; a
  first 401 refreshes and retries once. Every other failure (network,
  timeouts, 5xx, 420/429, other refresh errors) keeps the token and marks
  it failing (`failing_since`, `last_failure`), and the next success
  clears the mark. After 3 days of failing the character counts as
  lapsed for the warnings and the token mail. A 403 never touches the
  token; on the source read it sets `unreadable_since`.
- A failed read of a character's contacts skips its sync for that run.
  The ledger (`character_synced_contacts`) records contacts that already
  match plus writes ESI accepted; a refused add or edit keeps its old
  row, and a failed delete keeps its row for a later run. Only ledger
  contacts the source dropped are deleted in game; personal contacts are
  never touched.

## Testing rules

- Every change lands with its tests in the same commit.
- Tests are precise: exact JSON with keys in order at every level
  (`common::assert_json_eq`), exact messages, statuses and orderings,
  with the expected values written out in the test.
- Behavior tests drive the real router (`oneshot`) on the dedicated
  `bluebook_test` database (`db::test_pool()`, created and migrated on
  first use, `TEST_DATABASE_URL` to override) with a fixed clock
  (`clock::Clock`). ESI, EVE SSO and Discord are local mocks on ephemeral
  ports (`tests/integration/common/mocks.rs`), never stubs of our own
  code. The page, form and background suites start from a small seeded
  world (`common::world`); the others build what they need with the
  factories.
- Integration suites are modules of the single `tests/integration`
  binary and run single-threaded (`.cargo/config.toml`): they truncate
  the shared tables. Run one suite with `cargo test <module>::`.
- Frontend logic has vitest suites next to the code (`*.test.ts`, and
  `*.svelte.test.ts` in a browser). Playwright (`frontend/e2e`) runs
  against the composed stack and seeds through `docker exec
  bluebook-postgres psql`.
- Before committing: `cargo fmt`, `cargo clippy --all-targets` (zero
  warnings), `cargo test`, and in `frontend/` `npm run check`,
  `npm run lint`, `npm run format:check`, `npm test`.

## Layout and commands

- `src/auth/` the session, EVE SSO, the login and the ESI tokens;
  `src/esi.rs` the ESI client; `src/standings/` the source refresh, the
  character sync, the token check, the effective standings and the
  dashboard flags; `src/entities.rs` corporations, alliances and
  affiliations; `src/jobs.rs` the queue; `src/scheduler.rs` the schedule;
  `src/discord.rs` the webhooks; `src/server/` the router, the error
  answers and the page and form handlers.
- `migrations/` the schema (never edit an applied migration, sqlx checks
  their checksums); `deploy/` the Caddyfile and the proxy image.
- `frontend/` the SvelteKit app; `frontend/proxy-paths.ts` is the routing
  contract, mirrored by `deploy/Caddyfile`.
- Development: `docker compose up -d postgres`, `cargo run` (API on
  `BIND_ADDR`, default 127.0.0.1:3000) and `npm run dev` in `frontend/`.
  `.env` for config (see `.env.example`).
- Whole stack: `docker compose up --build` on http://localhost:5315, with
  the demo seeded (`/demo-login`). Ports are fixed: Postgres 5445, stack
  5315. The local preview is https://bluebook.test (portal).
- `cargo run --bin seed_demo` seeds the demo; `sync_standings [--force]`,
  `sync_affiliations` and `check_tokens` run the scheduled tasks once;
  `migrate` applies the migrations and exits.
- Production: Coolify runs `compose.prod.yml` (`bluebook-api`,
  `bluebook-frontend` and `bluebook-proxy`, images from GHCR built by
  `.github/workflows/release.yml`); the api reaches the shared Coolify
  Postgres over the `coolify` network.

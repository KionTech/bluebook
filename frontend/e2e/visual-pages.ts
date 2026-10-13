// The pages the visual comparison shoots, shared by the Laravel capture
// (scripts/laravel-reference/capture.mjs) and visual.spec.ts, so both
// sides shoot the same thing: tests/fixtures/dataset.json loaded at a
// fixed moment, the browser's clock at that moment, each page as a guest
// or one of the dataset's users, at desktop and phone width.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

/** The moment the dataset is loaded at and the browser's clock shows. */
export const VISUAL_NOW = '2026-06-25T14:00:00Z';

export const VIEWPORTS = {
  desktop: { width: 1280, height: 800 },
  phone: { width: 390, height: 844 },
} as const;

export type VisualPage = {
  name: string;
  path: string;
  /** The dataset's user to sign in as; a guest without. */
  user?: number;
};

export const VISUAL_PAGES: VisualPage[] = [
  { name: 'welcome', path: '/' },
  { name: 'dashboard', path: '/dashboard', user: 2 },
  { name: 'dashboard-wizard', path: '/dashboard', user: 3 },
  { name: 'appearance', path: '/settings/appearance', user: 2 },
  { name: 'admin', path: '/admin', user: 1 },
  { name: 'admin-requests', path: '/admin/standing-requests', user: 1 },
  { name: 'admin-pilots', path: '/admin/pilots', user: 1 },
  { name: 'admin-pilots-corporations', path: '/admin/pilots?view=corporations', user: 1 },
  { name: 'admin-settings', path: '/admin/settings', user: 1 },
];

type Row = Record<string, unknown>;

function literal(value: unknown): string {
  if (value === null || value === undefined) {
    return 'null';
  }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value);
  }
  return `'${String(value).replaceAll("'", "''")}'`;
}

function at(offset: unknown, now: Date): string {
  if (offset && typeof offset === 'object' && 'offset_minutes' in offset) {
    const minutes = Number((offset as { offset_minutes: number }).offset_minutes);
    return literal(new Date(now.getTime() + minutes * 60_000).toISOString());
  }
  return 'null';
}

/** The dataset as SQL for the stack's Postgres, the way record.py seeds
 * it into MySQL (and tests/integration/common/golden.rs into the test
 * database). */
export function datasetSql(now: Date = new Date(VISUAL_NOW)): string {
  const path = fileURLToPath(new URL('../../tests/fixtures/dataset.json', import.meta.url));
  const data = JSON.parse(readFileSync(path, 'utf8')) as Record<string, Row[]>;
  const stamp = literal(now.toISOString());
  const sql: string[] = [];
  for (const row of data.alliances) {
    sql.push(
      `insert into alliances (id, name, ticker, created_at, updated_at) values (${row.id}, ${literal(row.name)}, ${literal(row.ticker)}, ${stamp}, ${stamp})`,
    );
  }
  for (const row of data.corporations) {
    sql.push(
      `insert into corporations (id, name, ticker, alliance_id, created_at, updated_at) values (${row.id}, ${literal(row.name)}, ${literal(row.ticker)}, ${literal(row.alliance_id)}, ${stamp}, ${stamp})`,
    );
  }
  for (const row of data.factions) {
    sql.push(
      `insert into factions (id, name, created_at, updated_at) values (${row.id}, ${literal(row.name)}, ${stamp}, ${stamp})`,
    );
  }
  for (const row of data.users) {
    sql.push(
      `insert into users (id, name, setup_completed_at, created_at, updated_at) values (${row.id}, ${literal(row.name)}, ${at(row.setup_completed_at, now)}, ${stamp}, ${stamp})`,
    );
  }
  data.characters.forEach((row, index) => {
    const hash = row.user_id === null ? 'null' : literal(`owner-hash-${index + 1}`);
    sql.push(
      `insert into characters (id, name, corporation_id, alliance_id, user_id, character_owner_hash, should_sync, created_at, updated_at) values (${row.id}, ${literal(row.name)}, ${literal(row.corporation_id)}, ${literal(row.alliance_id)}, ${literal(row.user_id)}, ${hash}, ${literal(row.should_sync)}, ${stamp}, ${stamp})`,
    );
  });
  for (const row of data.users) {
    sql.push(
      `update users set main_character_id = ${literal(row.main_character_id)} where id = ${row.id}`,
    );
  }
  data.esi_tokens.forEach((row, index) => {
    const number = index + 1;
    const scopes = `array[${(row.scopes as string[]).map(literal).join(', ')}]::text[]`;
    sql.push(
      `insert into esi_tokens (id, character_id, access_token, token_type, refresh_token, character_owner_hash, scopes, expires_at, created_at, updated_at) values (${number}, ${row.character_id}, 'access-token-${number}', 'Bearer', 'refresh-token-${number}', 'owner-hash-${number}', ${scopes}, ${at(row.expires_at, now)}, ${stamp}, ${stamp})`,
    );
  });
  for (const row of data.standings_sources) {
    sql.push(
      `insert into standings_sources (type, entity_id, last_synced_at, created_at, updated_at) values (${literal(row.type)}, ${row.entity_id}, ${at(row.last_synced_at, now)}, ${stamp}, ${stamp})`,
    );
  }
  for (const row of data.source_contacts) {
    sql.push(
      `insert into source_contacts (contact_id, contact_type, standing, created_at, updated_at) values (${row.contact_id}, ${literal(row.contact_type)}, ${row.standing}, ${stamp}, ${stamp})`,
    );
  }
  for (const row of data.character_synced_contacts) {
    sql.push(
      `insert into character_synced_contacts (character_id, contact_id, standing, created_at, updated_at) values (${row.character_id}, ${row.contact_id}, ${row.standing}, ${stamp}, ${stamp})`,
    );
  }
  for (const row of data.standing_requests) {
    sql.push(
      `insert into standing_requests (id, subject_type, subject_id, requested_by_character_id, status, created_at, updated_at) values (${row.id}, ${literal(row.subject_type)}, ${row.subject_id}, ${row.requested_by_character_id}, ${literal(row.status)}, ${at(row.created_at, now)}, ${stamp})`,
    );
  }
  for (const row of data.discord_settings) {
    sql.push(
      `insert into discord_settings (webhook_url, role_id, created_at, updated_at) values (${literal(row.webhook_url)}, ${literal(row.role_id)}, ${stamp}, ${stamp})`,
    );
  }
  for (const table of ['users', 'esi_tokens', 'standing_requests']) {
    sql.push(
      `select setval(pg_get_serial_sequence('${table}', 'id'), (select max(id) from ${table}))`,
    );
  }
  return sql.join(';\n');
}

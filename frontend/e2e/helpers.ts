// Shared helpers for the e2e suite: everything talks to the docker stack,
// so the database is seeded and inspected through psql in the
// bluebook-postgres container. The stack's admin is character 2100000001
// (EVE_ADMIN_CHARACTER_ID in docker-compose.yml).
import { createHash, randomBytes } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { expect, type BrowserContext, type Page } from '@playwright/test';

/** The admin character of the composed stack. */
export const ADMIN_CHARACTER_ID = 2_100_000_001;

export function psql(sql: string): string {
  return execFileSync(
    'docker',
    ['exec', 'bluebook-postgres', 'psql', '-U', 'bluebook', '-d', 'bluebook', '-qtAc', sql],
    { encoding: 'utf8' },
  ).trim();
}

/** A SQL string literal. */
export function quote(value: string | null): string {
  return value === null ? 'null' : `'${value.replaceAll("'", "''")}'`;
}

/** Every table empty, ids from 1. */
export function resetDatabase(): void {
  psql(
    `truncate alliances, corporations, factions, users, characters, esi_tokens, source_contacts,
       character_synced_contacts, standings_sources, standing_requests, discord_settings,
       sessions, throttles, jobs restart identity cascade`,
  );
}

let sequence = 0;

/** A user; `setUp: false` for one who has not been through the setup. */
export function createUser(attributes: { name?: string; setUp?: boolean } = {}): number {
  sequence += 1;
  const name = attributes.name ?? `Test User ${sequence}`;
  const setUp = attributes.setUp ?? true;
  return Number(
    psql(
      `insert into users (name, setup_completed_at, created_at, updated_at)
       values (${quote(name)}, ${setUp ? 'now()' : 'null'}, now(), now()) returning id`,
    ),
  );
}

export type CharacterAttributes = {
  id?: number;
  name?: string;
  /** The owner; a new user unless given, `null` for none. */
  user?: number | null;
  corporationId?: number | null;
  allianceId?: number | null;
  shouldSync?: boolean;
};

/** A character; unset fields get defaults. */
export function createCharacter(attributes: CharacterAttributes = {}): number {
  sequence += 1;
  const id = attributes.id ?? 95_000_000 + sequence;
  const name = attributes.name ?? `Pilot ${sequence}`;
  const user = attributes.user === undefined ? createUser() : attributes.user;
  psql(
    `insert into characters (id, name, user_id, corporation_id, alliance_id, character_owner_hash,
       should_sync, created_at, updated_at)
     values (${id}, ${quote(name)}, ${user ?? 'null'}, ${attributes.corporationId ?? 'null'},
       ${attributes.allianceId ?? 'null'}, 'owner-${sequence}', ${attributes.shouldSync ?? false},
       now(), now())`,
  );
  return id;
}

/** `$user->mainCharacter()->associate($character)->save()`. */
export function setMain(user: number, character: number): void {
  psql(`update users set main_character_id = ${character} where id = ${user}`);
}

/** `grantScopes($character, ...$scopes)`. */
export function grantScopes(character: number, scopes: string[]): void {
  const array = `array[${scopes.map(quote).join(', ')}]::text[]`;
  psql(
    `insert into esi_tokens (character_id, access_token, token_type, refresh_token,
       character_owner_hash, scopes, expires_at, created_at, updated_at)
     values (${character}, 'access', 'Bearer', 'refresh', 'owner', ${array},
       now() + interval '1 hour', now(), now())`,
  );
}

export function createCorporation(
  id: number,
  name: string,
  ticker: string | null = null,
  allianceId: number | null = null,
): void {
  psql(
    `insert into corporations (id, name, ticker, alliance_id, created_at, updated_at)
     values (${id}, ${quote(name)}, ${quote(ticker)}, ${allianceId ?? 'null'}, now(), now())`,
  );
}

export function createAlliance(id: number, name: string, ticker: string | null = null): void {
  psql(
    `insert into alliances (id, name, ticker, created_at, updated_at)
     values (${id}, ${quote(name)}, ${quote(ticker)}, now(), now())`,
  );
}

/** `StandingsSource::create([...])`; `syncedMinutesAgo` sets `last_synced_at`. */
export function createSource(type: string, entityId: number, syncedMinutesAgo?: number): void {
  const synced =
    syncedMinutesAgo === undefined ? 'null' : `now() - interval '${syncedMinutesAgo} minutes'`;
  psql(
    `insert into standings_sources (type, entity_id, last_synced_at, created_at, updated_at)
     values (${quote(type)}, ${entityId}, ${synced}, now(), now())`,
  );
}

export function createSourceContact(
  contactType: string,
  contactId: number,
  standing: number,
): void {
  psql(
    `insert into source_contacts (contact_id, contact_type, standing, created_at, updated_at)
     values (${contactId}, ${quote(contactType)}, ${standing}, now(), now())`,
  );
}

/** A standing request by the character. */
export function createStandingRequest(
  character: number,
  attributes: { status?: string; subjectType?: string; subjectId?: number } = {},
): number {
  return Number(
    psql(
      `insert into standing_requests (subject_type, subject_id, requested_by_character_id, status,
         created_at, updated_at)
       values (${quote(attributes.subjectType ?? 'character')}, ${attributes.subjectId ?? character},
         ${character}, ${quote(attributes.status ?? 'pending')}, now(), now()) returning id`,
    ),
  );
}

/** A user owning the stack's admin character. */
export function createAdmin(name = 'Admin Pilot'): number {
  const user = createUser();
  createCharacter({ id: ADMIN_CHARACTER_ID, name, user });
  return user;
}

/** Signs the user in: a session for the user, its cookie in the
 * browser. */
export async function signIn(
  context: BrowserContext,
  user: number,
  baseURL: string,
): Promise<void> {
  const token = randomBytes(30).toString('hex');
  const hash = createHash('sha256').update(token).digest('hex');
  psql(
    `insert into sessions (id, user_id, data, last_activity, created_at)
     values ('${hash}', ${user}, '{}', now(), now())`,
  );
  const url = new URL(baseURL);
  await context.addCookies([
    {
      name: 'bluebook-session',
      value: token,
      domain: url.hostname,
      path: '/',
      httpOnly: true,
      sameSite: 'Lax',
    },
  ]);
}

/** No JavaScript errors and no console errors
 * on the page, apart from the browser's own line for a 422 answer (a
 * form with errors). */
export function watchForErrors(page: Page): () => void {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error' && !message.text().includes('422')) {
      errors.push(message.text());
    }
  });
  return () => expect(errors).toEqual([]);
}

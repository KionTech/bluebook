# Fixes

Every Laravel bug the rewrite fixes instead of copying, with the test that
pins the new behavior, then the deliberate changes and what is left for
the owner to decide. Together with the test normalizations
(`tests/integration/common/golden.rs`) this is the complete list of
differences from the Laravel app on `main`.

## Bugs fixed

### Removed characters were mailed about lapsed tokens forever

Laravel: removing a character from an account deleted its tokens but left
`should_sync` on, and the daily token check mails every `should_sync`
character without a write token (once every three days). A removed
character, which nobody can grant a token for any more, got the
"re-authentication required" EVE mail every three days for good.

Now: removing a character turns its sync off, and the token check only
mails characters that belong to an account.

Tests: `check_tokens::does_not_mail_characters_without_an_account`,
`characters::removing_a_character_turns_its_sync_off`.

### The "source unreadable" alert linked a login without scopes

Laravel: the Discord alert's "Click here to re-authenticate" link was the
plain login (`/eve`), which asks EVE for no scopes, so following it never
gave the admin character the contact scopes the source needs.

Now: the link is the admin grant link (the admin scopes, added to the
account), the same one the banner on the site uses.

Test: `standings_sync::the_unreadable_alert_links_the_admin_grant`.

### "Sync now" said the sync started when the source could not be read

Laravel: the admin's "Sync now" ran the source refresh inside the request
and always flashed "Standings sync started.", also when no admin token
could read the source and nothing happened.

Now: an unreadable source flashes the error "The standings source could
not be read." instead; a successful run (or a skip during EVE's downtime)
keeps the old message.

Test: `admin_source::sync_now_reports_an_unreadable_source`.

### An unknown scope in the login link was a server error

Laravel: `/eve?scopes=...` with a value that is not an ESI scope threw in
`EsiScope::from` and answered 500.

Now: 400 Bad Request, and nothing is stored.

Test: `login_scopes::an_unknown_scope_is_a_bad_request`.

### Every scopeless login stored a token that was never used

Laravel: each login created an `esi_tokens` row, also when EVE granted no
scope at all (the normal login asks for none). Nothing ever reads a token
without scopes, and the rows were never deleted.

Now: a login without scopes stores no token; scoped logins store theirs
as before.

Test: `login::a_login_without_scopes_stores_no_token`.

### Percent and underscore in the pilot search were wildcards

Laravel: the admin pilot search put the term into `LIKE '%term%'`
unescaped, so `%` matched everything and `_` any character.

Now: the term is matched literally (still case-insensitive, still on the
name, the ticker and the member characters' names).

Test: `admin_pilots::the_search_matches_percent_and_underscore_literally`.

### A failed token refresh still sent the request with the dead token

Laravel: when the refresh of an expiring token failed (and the token row
was deleted), the paginated reads (contacts) still went out with the old
access token, and failed. Single requests did check the failure first.

Now: a failed refresh is the answer, no request goes out. The outcome
(a failed read, the token gone) is the same.

Test: `esi_tokens::a_failed_refresh_sends_no_request`.

## Deliberate changes

### One new login instead of a session bridge

Sessions and remember cookies of the Laravel app are not carried over:
everyone logs in again once with EVE SSO (one click for an existing CCP
session). The new session lives in the `bluebook-session` cookie (a random
token, stored hashed) and lasts `SESSION_LIFETIME_DAYS` (400, the old
remember cookie's lifetime). ESI tokens are kept, so syncing continues.

Tests: `sessions::a_session_lasts_its_lifetime`,
`login::a_login_starts_a_session`.

### Forms answer JSON

Laravel redirected back after every form and flashed the message into the
session. The pages now send the forms with `fetch` and get
`200 {"redirect": path | null}` (null: stay and reload, Laravel's
`back()`), `422 {"errors": {field: first message}}`, `401` with the login
for a guest, `403` and `404`. The flash message still travels in the
session and arrives with the next page's props (`flash`).

Tests: the suites `characters`, `standing_requests`, `admin`, `admin_source`.

### Cross-site forms are refused without a CSRF token

There is no CSRF token any more. A mutation the browser marks as coming
from another site (`Sec-Fetch-Site`) or whose `Origin` names another host
answers 419 like a stale CSRF token did. A guest's form answers 401 with
the login (Laravel: 302 to `/eve`, and 419 for a guest's logout, because
the CSRF check ran first).

Tests: `sessions::a_cross_site_form_is_refused`,
`sessions::a_guest_form_is_sent_to_the_login`.

### Orderings without an `ORDER BY`

Laravel relied on MySQL's primary key order where it gave none: the
characters of an account (the first one becomes active), the admin
character that reads the source, the dashboard's characters. They are
ordered by id now. Requests with the same `created_at` are ordered by id,
newest first; standings with the same value and type by id.

Tests: `dashboard::characters_are_listed_by_id`,
`standings_source::the_first_admin_by_id_reads_the_source`.

### The ESI user agent names the site

Laravel sent `User-Agent: Bluebook`. CCP asks for a user agent with a
way to reach the developer; it is `Bluebook (+https://bluebook.nicolaskion.dev)`
now (`ESI_USER_AGENT`).

Test: `esi::requests_carry_the_user_agent_and_compatibility_date`.

### The schedule is off unless switched on

The standings sync, the affiliation refresh and the token check run only
with `SCHEDULER_ENABLED`, so a local stack, a staging copy or a second
deployment never writes to the pilots' contacts or refreshes their tokens
behind the live site's back.

Test: `scheduler::the_schedule_is_off_unless_enabled`.

### A trailing slash redirects

Laravel served `/dashboard/` like `/dashboard`. The pages answer a 308
to the address without the slash now (SvelteKit's default, as on the
other rewritten sites); the forms still take both.

Tests: `frontend/e2e/routes.spec.ts` "a trailing slash redirects to the
page without it", `pages::a_trailing_slash_reaches_the_same_form`.

### The demo login

`/demo-login` signs in as a demo pilot without EVE SSO when
`DEMO_ENABLED` is set (the local stack; never production). The home page
then shows a link to it. With `?user=<id>` it signs in as that user, which
the e2e suite uses.

Tests: `demo::the_demo_login_is_off_by_default`,
`demo::the_demo_login_signs_in_the_demo_pilot`.

## Needs owner decision

Kept as Laravel has it, listed for the owner:

- The destructive token rules: a failed refresh (any status, also a 5xx
  that survived the retries) deletes the token, and any ESI 401 or 403 on
  an authenticated call deletes the token used. A corporation source read
  by a character that lacks the in-game role answers 403 and so kills the
  admin's token.
- A failed read of a character's contacts counts as "no contacts", so the
  sync adds every standing again; and the ledger of what was written is
  set to the source's list even when the writes failed.
- The admin settings page and "Set source" use the active character, not
  necessarily an admin character: an admin whose active character is an
  alt sets the source from the alt's corporation or alliance, which no
  admin character may be able to read.
- The contact address in the ESI user agent (now the site's URL).
- `factions` (the names of faction contacts) is static SDE data, imported
  once and never refreshed.
- Asking again for a rejected standing reopens the old request, which keeps
  its original date, so it sorts among the old requests.
- The tab titles end in the app name, `APP_NAME` ("Bluebook"), as the
  production build's `VITE_APP_NAME="${APP_NAME}"` gave them; a build
  without it said "Standings". Confirm the production value.

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

### The admin settings page lost the active character's tickers

Laravel: the settings controller loaded the active character's
corporation and alliance with only their id and name, before the shared
props loaded them with their tickers, which then counted as loaded
already. On `/admin/settings` the active character (in
`auth.active_character` and in `auth.characters`) had `ticker: null`
for its corporation and alliance.

Now: the tickers are there on every page.

Test: `golden_pages::admin_settings` (and `admin_settings_alt`,
`admin_settings_no_source`), which expect the dataset's tickers.

### A failed token refresh still sent the request with the dead token

Laravel: when the refresh of an expiring token failed (and the token row
was deleted), the paginated reads (contacts) still went out with the old
access token, and failed. Single requests did check the failure first.

Now: a failed refresh is the answer, no request goes out. The outcome
(a failed read, the token gone) is the same.

Test: `esi_tokens::a_failed_refresh_sends_no_request`.

### Tokens were thrown away on any failure

Laravel: a failed refresh deleted the token whatever the reason (any
error status, also a 5xx that survived the five tries), and any ESI 401
or 403 on an authenticated call deleted the token used. An SSO outage or a
rate limit cost every pilot their tokens, and a corporation source read by
a character without the in-game role (403) killed the admin's token.

Now a token is deleted only when it is certainly dead:

- the refresh is answered 400 or 401 with the OAuth error `invalid_grant`
  (or `invalid_token`), EVE's answer for a revoked or expired refresh
  token;
- ESI answers 401 right after a fresh refresh. A 401 on a token that was
  not just refreshed refreshes it and tries once more.

Everything else keeps the token: an unreachable SSO or ESI, timeouts,
5xx after the tries, 420 and 429, other refresh errors (`invalid_client`,
`invalid_request`). The token is marked failing (`failing_since`, the
first failure in a row, and `last_failure`) and tried again on the next
run; the next success clears the mark. A write token failing for three
days counts as lapsed: the character is in `reauth_characters` and gets
the token mail, but the token stays. A 403 never touches the token; on
the source read it marks the source unreadable (`unreadable_since`, the
admin banner and the Discord alert) until a read works again.

Tests: `esi_tokens::a_refresh_answered_with_invalid_grant_deletes_the_token`,
`esi_tokens::a_refresh_failing_five_times_keeps_the_token`,
`esi_tokens::other_refresh_errors_keep_the_token`,
`esi_tokens::an_unreachable_sso_keeps_the_token`,
`esi_tokens::a_refresh_that_works_again_clears_the_failure_mark`,
`esi::a_forbidden_answer_keeps_the_token_and_marks_the_source_unreadable`,
`esi::a_read_that_works_again_clears_the_unreadable_mark`,
`esi::an_unauthorized_answer_refreshes_the_token_and_tries_again`,
`esi::an_unauthorized_answer_right_after_a_refresh_deletes_the_token`,
`esi::transient_failures_keep_the_token_and_mark_it_failing`,
`esi::an_unreachable_esi_keeps_the_token`,
`esi::a_success_clears_the_failure_mark_but_keeps_its_start_while_failing`,
`check_tokens::mails_characters_whose_write_token_has_failed_for_three_days`,
`access_warnings::flags_a_character_whose_write_token_has_failed_for_three_days`,
`golden_background::sync_forbidden` (expects the tokens kept).

### A failed read of a character's contacts counted as no contacts

Laravel: when the character's contacts could not be read (ESI down, a
refused token), the sync went on as if the character had none: it sent
every standing again as new contacts, and never deleted stale ones.

Now: a failed read stops that character's sync for this run; nothing is
written and its ledger stays as it was. A character without the read
scope still gets every standing written, as before.

Tests: `character_sync::a_failed_read_of_the_contacts_aborts_the_sync`,
`golden_background::character_sync_unreadable` (expects only the read).

### The ledger recorded writes ESI refused

Laravel: after the writes, the ledger of synced contacts
(`character_synced_contacts`) was set to the source's list, also for adds
and edits ESI refused, so a contact that never reached the game counted as
synced, and a stale one whose delete failed was forgotten and never
deleted.

Now the ledger records what is in game because of the sync: contacts that
already match, adds and edits ESI accepted, and contacts whose delete
failed (deleted on a later run). A refused add or edit keeps the row it
had before.

Tests: `character_sync::the_ledger_records_only_writes_that_succeeded`,
`character_sync::a_failed_delete_keeps_its_ledger_row`,
`character_sync::a_successful_delete_removes_its_ledger_row`.

### The source came from the active character, not an admin character

Laravel: the admin settings page and "Set source" used the active
character. An admin whose active character was an alt set the source from
the alt's corporation or alliance (or got "Your character has no alliance
to use as a source."), which no admin character might be able to read.

Now both use an admin character of the account: the active one first,
then by id, the first that holds the source type's read scope and has
that entity; without one, the active character when it is an admin, else
the first admin character. The mail scope shown is the admin character's
the same way.

Tests: `admin_source::uses_an_admin_character_while_an_alt_is_active`,
`admin_source::prefers_the_admin_character_that_can_read_the_source`,
`admin_source::keeps_the_active_admin_character_when_no_one_can_read_yet`,
`admin_settings::offers_the_admin_characters_entities_while_an_alt_is_active`,
`golden_pages::admin_settings_alt`, `golden_mutations::admin_source_no_alliance`.

### Requests named the requester's corporation, not the one asked for

Laravel: a corporation or alliance request took its name from the
requesting character's current corporation or alliance. Once the
requester moved, the admin requests page and the Discord announcement
named the wrong entity, or showed only the id.

Now the name is the subject's own, looked up by its id (a character
request still names the requesting character, which asks for itself).

Tests: `admin::names_a_request_by_its_subject_after_the_requester_moved`,
`standing_requests::announces_a_request_by_its_subject_after_the_requester_moved`.

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

- The contact address in the ESI user agent (now the site's URL).
- `factions` (the names of faction contacts) is static SDE data, imported
  once and never refreshed.
- Asking again for a rejected standing reopens the old request, which keeps
  its original date, so it sorts among the old requests.
- The tab titles end in the app name, `APP_NAME` ("Bluebook"), as the
  production build's `VITE_APP_NAME="${APP_NAME}"` gave them; a build
  without it said "Standings". Confirm the production value.

-- Transient failures no longer delete a token: the first failure in a row
-- is remembered (and cleared by the next success), so a token that keeps
-- failing for days can be reported without being thrown away.
alter table esi_tokens
    add column failing_since timestamptz,
    add column last_failure text;

-- A source that could not be read (an admin character without the in-game
-- role answers 403, ESI is down) stays marked until a read works again.
alter table standings_sources
    add column unreadable_since timestamptz;

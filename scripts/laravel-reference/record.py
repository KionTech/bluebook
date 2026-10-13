#!/usr/bin/env python3
"""Records golden fixtures from the running Laravel reference.

    scripts/laravel-reference/run.sh start
    python3 -I scripts/laravel-reference/record.py
    python3 -I scripts/laravel-reference/record.py seed   (the dataset only)
    python3 -I scripts/laravel-reference/record.py pages|mutations|background

Every scenario starts from tests/fixtures/dataset.json (times relative to
the moment of the recording), with its outgoing HTTP calls answered from
the scenario's fakes. Writes tests/fixtures/golden/:

- pages/<name>.json: one GET as a user (or a guest): the status, the
  redirect target or the Inertia page object (component and props), or
  the HTML of a page that is not Inertia's.
- mutations/<name>.json: a form as a user: its status and redirect, the
  flash message and errors the next page shows, the rows it left behind,
  the jobs it queued and the HTTP calls it made.
- background/<name>.json: an artisan command or a job: its exit code, the
  jobs it queued, the HTTP calls (with the fakes that answered them) and
  the rows it left behind.
- meta.json: the moment of the recording, so the Rust suites can pin
  their clock to it.

Only the standard library; run with `python3 -I`.
"""

import datetime
import http.cookiejar
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request

BASE = "http://127.0.0.1:8315"
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
REF_DIR = os.environ.get("LARAVEL_REF_DIR", "/tmp/bluebook-laravel-ref")
APP = os.path.join(REF_DIR, "app")
OUT = os.path.join(REPO, "tests/fixtures/golden")
RUN = os.path.join(REPO, "scripts/laravel-reference/run.sh")
DATASET = json.load(open(os.path.join(REPO, "tests/fixtures/dataset.json"), encoding="utf-8"))
# RECORD_NOW (ISO 8601) pins the moment, so a re-recording matches the fixtures' times.
NOW = (
    datetime.datetime.fromisoformat(os.environ["RECORD_NOW"].replace("Z", "+00:00"))
    if os.environ.get("RECORD_NOW")
    else datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0)
)
ESI = "https://esi.evetech.net"
SSO = "https://login.eveonline.com"
WEBHOOK = "https://discord.test/webhook"

ADMIN = 2112000001
ADMIN_ALT = 2112000002
NICOLAS = 95000001
TINA = 95000002
EMILE = 95000003
FRESH = 95000010
RED = 95000020
MEMBER = 95000030

ADMIN_SCOPES = ",".join([
    "publicData",
    "esi-characters.read_contacts.v1",
    "esi-characters.write_contacts.v1",
    "esi-corporations.read_contacts.v1",
    "esi-alliances.read_contacts.v1",
    "esi-mail.send_mail.v1",
])


def resolve(value):
    """Dataset times: `{offset_minutes}` from NOW, as MySQL writes them."""
    if isinstance(value, dict) and "offset_minutes" in value:
        moment = NOW + datetime.timedelta(minutes=value["offset_minutes"])
        return moment.strftime("%Y-%m-%d %H:%M:%S")
    return value


def sql(statement):
    return subprocess.run([RUN, "sql", statement], check=True, capture_output=True, text=True).stdout


def artisan(*args):
    result = subprocess.run(
        ["docker", "exec", "bluebook-laravel-ref", "php", "artisan", *args],
        capture_output=True,
        text=True,
    )
    return result.returncode, result.stdout


def php(code):
    out = subprocess.run([RUN, "php", code], check=True, capture_output=True, text=True).stdout
    return out.strip()


def literal(value):
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "1" if value else "0"
    if isinstance(value, (int, float)):
        return repr(value)
    return "'" + str(value).replace("\\", "\\\\").replace("'", "\\'") + "'"


def insert(table, row):
    row = {key: resolve(value) for key, value in row.items()}
    columns = ", ".join(f"`{key}`" for key in row)
    values = ", ".join(literal(value) for value in row.values())
    return f"insert into {table} ({columns}) values ({values})"


TRUNCATE = [
    "users", "characters", "corporations", "alliances", "factions", "esi_tokens",
    "esi_token_scope", "source_contacts", "character_synced_contacts", "standings_sources",
    "standing_requests", "discord_settings", "sessions", "cache", "cache_locks", "jobs",
    "failed_jobs",
]


def seed(extra_sql=()):
    now = resolve({"offset_minutes": 0})
    stamps = {"created_at": now, "updated_at": now}
    statements = ["set foreign_key_checks = 0"]
    statements += [f"truncate table {table}" for table in TRUNCATE]
    for row in DATASET["alliances"]:
        statements.append(insert("alliances", {**row, **stamps}))
    for row in DATASET["corporations"]:
        statements.append(insert("corporations", {**row, **stamps}))
    for row in DATASET["factions"]:
        statements.append(insert("factions", {
            **row, "description": "", "size_factor": 5, "station_count": 0,
            "station_system_count": 0, "is_unique": 1, **stamps,
        }))
    for row in DATASET["users"]:
        statements.append(insert("users", {**row, **stamps}))
    for number, row in enumerate(DATASET["characters"], 1):
        statements.append(insert("characters", {
            **row, "character_owner_hash": None if row["user_id"] is None else f"owner-hash-{number}",
            **stamps,
        }))
    for number, row in enumerate(DATASET["esi_tokens"], 1):
        statements.append(insert("esi_tokens", {
            "id": number, "character_id": row["character_id"], "access_token": f"access-token-{number}",
            "token_type": "Bearer", "refresh_token": f"refresh-token-{number}",
            "character_owner_hash": f"owner-hash-{number}", "expires_at": row["expires_at"], **stamps,
        }))
        for scope in row["scopes"]:
            statements.append(
                f"insert into esi_token_scope (esi_token_id, esi_scope_id) "
                f"select {number}, id from esi_scopes where name = {literal(scope)}"
            )
    for row in DATASET["standings_sources"]:
        statements.append(insert("standings_sources", {**row, **stamps}))
    for row in DATASET["source_contacts"]:
        statements.append(insert("source_contacts", {**row, **stamps}))
    for row in DATASET["character_synced_contacts"]:
        statements.append(insert("character_synced_contacts", {**row, **stamps}))
    for row in DATASET["standing_requests"]:
        statements.append(insert("standing_requests", {"updated_at": now, **row}))
    for row in DATASET["discord_settings"]:
        statements.append(insert("discord_settings", {**row, **stamps}))
    statements.append("set foreign_key_checks = 1")
    statements += [resolve_sql(statement) for statement in extra_sql]
    sql(";\n".join(statements))


def resolve_sql(statement):
    """`{minutes:N}` in a scenario's SQL is NOW plus N minutes."""
    return re.sub(
        r"\{minutes:(-?\d+)\}",
        lambda match: literal(resolve({"offset_minutes": int(match.group(1))})),
        statement,
    )


# ---------------------------------------------------------------------------
# HTTP
# ---------------------------------------------------------------------------


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


class Browser:
    def __init__(self):
        self.jar = http.cookiejar.CookieJar()
        self.opener = urllib.request.build_opener(
            NoRedirect, urllib.request.HTTPCookieProcessor(self.jar)
        )

    def request(self, method, path, body=None, headers=None):
        data = None if body is None else json.dumps(body).encode()
        request = urllib.request.Request(BASE + path, data=data, method=method, headers=headers or {})
        try:
            with self.opener.open(request) as response:
                return response.status, dict(response.headers), response.read().decode()
        except urllib.error.HTTPError as error:
            return error.code, dict(error.headers), error.read().decode()

    def login(self, user, character=None):
        query = f"?character={character}" if character else ""
        status, _, body = self.request("GET", f"/_reference/login/{user}{query}")
        assert status == 204, (status, body)

    def xsrf(self):
        for cookie in self.jar:
            if cookie.name == "XSRF-TOKEN":
                return urllib.parse.unquote(cookie.value)
        return ""


def page_object(html):
    match = re.search(r'<script data-page="app" type="application/json">(.*?)</script>', html, re.S)
    return json.loads(match.group(1)) if match else None


def site_path(location):
    if location.startswith(BASE):
        return location[len(BASE):] or "/"
    return None


# ---------------------------------------------------------------------------
# Fakes, rows, jobs
# ---------------------------------------------------------------------------


def write_fakes(fakes):
    with open(os.path.join(APP, "storage/app/http-fakes.json"), "w", encoding="utf-8") as file:
        json.dump(fakes, file)
    log = os.path.join(APP, "storage/logs/http.jsonl")
    if os.path.exists(log):
        os.remove(log)


def http_log():
    log = os.path.join(APP, "storage/logs/http.jsonl")
    if not os.path.exists(log):
        return []
    calls = []
    for line in open(log, encoding="utf-8"):
        entry = json.loads(line)
        headers = {key.lower(): value for key, value in entry["headers"].items()}
        try:
            body = json.loads(entry["body"]) if entry["body"] else None
        except ValueError:
            body = entry["body"]
        calls.append({
            "method": entry["method"],
            "url": entry["url"],
            "headers": {
                key: headers[key]
                for key in ["user-agent", "x-compatibility-date", "authorization", "content-type", "host"]
                if key in headers
            },
            "body": body,
        })
    return calls


DUMPS = {
    "users": "select json_object('id', id, 'name', name, 'main_character_id', main_character_id, "
    "'set_up', setup_completed_at is not null) from users order by id",
    "characters": "select json_object('id', id, 'name', name, 'corporation_id', corporation_id, "
    "'alliance_id', alliance_id, 'user_id', user_id, 'should_sync', should_sync, "
    "'has_owner_hash', character_owner_hash is not null) from characters order by id",
    "corporations": "select json_object('id', id, 'name', name, 'ticker', ticker, "
    "'alliance_id', alliance_id, 'member_count', member_count) from corporations order by id",
    "alliances": "select json_object('id', id, 'name', name, 'ticker', ticker) from alliances order by id",
    "esi_tokens": "select json_object('character_id', t.character_id, 'access_token', t.access_token, "
    "'refresh_token', t.refresh_token, 'scopes', (select json_arrayagg(s.name) from esi_token_scope ts "
    "join esi_scopes s on s.id = ts.esi_scope_id where ts.esi_token_id = t.id)) "
    "from esi_tokens t order by t.character_id, t.id",
    "source_contacts": "select json_object('contact_id', contact_id, 'contact_type', contact_type, "
    "'standing', cast(standing as double)) from source_contacts order by contact_id",
    "character_synced_contacts": "select json_object('character_id', character_id, 'contact_id', "
    "contact_id, 'standing', cast(standing as double)) from character_synced_contacts "
    "order by character_id, contact_id",
    "standings_sources": "select json_object('type', type, 'entity_id', entity_id, 'synced', "
    "last_synced_at is not null) from standings_sources order by id",
    "standing_requests": "select json_object('id', id, 'subject_type', subject_type, 'subject_id', "
    "subject_id, 'requested_by_character_id', requested_by_character_id, 'status', status) "
    "from standing_requests order by id",
    "discord_settings": "select json_object('webhook_url', webhook_url, 'role_id', role_id) "
    "from discord_settings order by id",
    "throttles": "select json_object('key', replace(`key`, 'bluebook-cache-', '')) from cache "
    "where `key` like 'bluebook-cache-standings:%' order by `key`",
}

BOOLEANS = {"should_sync", "set_up", "has_owner_hash", "synced"}


def rows():
    tables = {}
    for table, query in DUMPS.items():
        lines = [line for line in sql(query).splitlines() if line.strip()]
        values = []
        for line in lines:
            row = json.loads(line)
            for key in BOOLEANS & row.keys():
                row[key] = bool(row[key])
            if row.get("scopes") is not None:
                row["scopes"] = sorted(row["scopes"])
            elif "scopes" in row:
                row["scopes"] = []
            values.append(row)
        tables[table] = values
    return tables


def jobs():
    """The queued jobs, as the Rust queue names them."""
    queued = []
    for line in sql("select payload from jobs order by id").splitlines():
        if not line.strip():
            continue
        payload = json.loads(line)
        name = payload["displayName"]
        command = payload["data"]["command"]
        if name == "App\\Jobs\\SyncCharacterStandings":
            character = re.search(r'Models\\Character";s:2:"id";i:(\d+)', command).group(1)
            queued.append({"kind": "character_sync", "character_id": int(character)})
        elif name == "App\\Jobs\\NotifyStandingRequest":
            request = re.search(r'Models\\StandingRequest";s:2:"id";i:(\d+)', command).group(1)
            queued.append({"kind": "discord_request", "standing_request_id": int(request)})
        elif name == "App\\Jobs\\NotifySourceUnreadable":
            queued.append({"kind": "discord_unreadable"})
        elif name == "App\\Jobs\\SendTokenExpiredMail":
            sender = re.search(r's:8:"senderId";i:(\d+)', command).group(1)
            recipient = re.search(r's:11:"recipientId";i:(\d+)', command).group(1)
            queued.append({"kind": "token_mail", "sender_id": int(sender), "recipient_id": int(recipient)})
        else:
            queued.append({"kind": name})
    return queued


def failed_jobs():
    return int(sql("select count(*) from failed_jobs").strip() or 0)


def save(kind, name, data):
    path = os.path.join(OUT, kind, f"{name}.json")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as file:
        json.dump(data, file, indent=2, ensure_ascii=False)
        file.write("\n")
    print(f"{kind}/{name}")


# ---------------------------------------------------------------------------
# ESI answers
# ---------------------------------------------------------------------------


def contact(contact_id, contact_type, standing):
    return {"contact_id": contact_id, "contact_type": contact_type, "standing": standing, "label_ids": []}


def corporation_payload(name, ticker, alliance_id=None):
    return {
        "name": name, "ticker": ticker, "ceo_id": 90000001, "creator_id": 90000001,
        "date_founded": "2015-03-01T12:00:00Z", "description": "", "member_count": 12,
        "tax_rate": 0.1, "url": "", "alliance_id": alliance_id, "shares": 1000,
        "war_eligible": True,
    }


def alliance_payload(name, ticker):
    return {
        "name": name, "ticker": ticker, "creator_corporation_id": 98000001,
        "creator_id": 90000001, "date_founded": "2012-05-01T00:00:00Z",
        "executor_corporation_id": 98000001,
    }


# The source after a change: two new contacts, one standing changed, two
# dropped, and the fresh pilot made blue.
SOURCE_CHANGED = [
    contact(99000001, "alliance", 10),
    contact(98000002, "corporation", 10),
    contact(95000100, "character", 5),
    contact(95000200, "character", 10),
    contact(98000006, "corporation", 5),
    contact(99000004, "alliance", -10),
    contact(98000003, "corporation", -5),
    contact(500001, "faction", -5),
    contact(99000003, "alliance", -10),
    contact(95000101, "character", -10),
    contact(98000004, "corporation", -5),
    contact(FRESH, "character", 7.5),
]

SOURCE_UNCHANGED = [
    contact(row["contact_id"], row["contact_type"], row["standing"]) for row in DATASET["source_contacts"]
]


def source_fakes(contacts):
    return {
        f"GET {ESI}/alliances/99000002/contacts/": {"body": contacts, "headers": {"X-Pages": "1"}},
        f"POST {ESI}/universe/names/": {"body": [
            {"category": "character", "id": 95000100, "name": "Contact One"},
            {"category": "character", "id": 95000200, "name": "New Friend"},
            {"category": "character", "id": 95000101, "name": "Spy Pilot"},
            {"category": "character", "id": FRESH, "name": "Fresh Pilot"},
        ]},
        f"POST {ESI}/characters/affiliation/": {"body": [
            {"character_id": 95000100, "corporation_id": 98000002, "alliance_id": 99000001},
            {"character_id": 95000200, "corporation_id": 98000007},
            {"character_id": 95000101, "corporation_id": 98000004, "alliance_id": 99000003},
            {"character_id": FRESH, "corporation_id": 98000003},
        ]},
        f"GET {ESI}/corporations/98000006/": {"body": corporation_payload("Sixth Corp", "SIX", 99000004)},
        f"GET {ESI}/corporations/98000007/": {"body": corporation_payload("Seventh Corp", "SEVEN")},
        f"GET {ESI}/alliances/99000004/": {"body": alliance_payload("Far Alliance", "FAR")},
    }


NICOLAS_CONTACTS = {
    f"GET {ESI}/characters/{NICOLAS}/contacts/": {"body": [
        contact(99000001, "alliance", 10),
        contact(98000002, "corporation", 5),
        contact(12345, "character", 5),
        contact(55555, "character", -10),
    ], "headers": {"X-Pages": "1"}},
}


# ---------------------------------------------------------------------------
# Scenarios
# ---------------------------------------------------------------------------

PAGES = [
    # (name, user, active character, path, extra SQL)
    ("home_guest", None, None, "/", []),
    ("dashboard_guest", None, None, "/dashboard", []),
    ("admin_guest", None, None, "/admin", []),
    ("settings_guest", None, None, "/settings", []),
    ("appearance_guest", None, None, "/settings/appearance", []),
    ("login", None, None, "/eve", []),
    ("login_scopes", None, None, "/eve?add_to_account=1&scopes=esi-characters.read_contacts.v1,esi-characters.write_contacts.v1", []),
    ("login_unknown_scope", None, None, "/eve?scopes=nope", []),
    ("callback_without_state", None, None, "/eve/callback", []),
    ("up", None, None, "/up", []),
    ("storage", None, None, "/storage/file.txt", []),
    ("not_found", None, None, "/nope", []),
    ("home_pilot", 2, NICOLAS, "/", []),
    ("dashboard_pilot", 2, NICOLAS, "/dashboard", []),
    ("dashboard_pilot_alt", 2, TINA, "/dashboard", []),
    ("dashboard_pilot_no_active", 2, None, "/dashboard", []),
    ("appearance_pilot", 2, NICOLAS, "/settings/appearance", []),
    ("settings_pilot", 2, NICOLAS, "/settings", []),
    ("admin_pilot", 2, NICOLAS, "/admin", []),
    ("admin_pilots_pilot", 2, NICOLAS, "/admin/pilots", []),
    ("dashboard_fresh", 3, FRESH, "/dashboard", []),
    ("dashboard_red", 4, RED, "/dashboard", []),
    ("dashboard_member", 5, MEMBER, "/dashboard", []),
    ("dashboard_no_source", 2, NICOLAS, "/dashboard", ["delete from standings_sources"]),
    ("dashboard_character_source", 2, NICOLAS, "/dashboard", [
        "update standings_sources set type = 'character', entity_id = 2112000001",
    ]),
    ("dashboard_admin", 1, ADMIN, "/dashboard", []),
    ("admin_overview", 1, ADMIN, "/admin", []),
    ("admin_overview_no_source", 1, ADMIN, "/admin", ["delete from standings_sources"]),
    ("admin_overview_unreadable", 1, ADMIN, "/admin", ["delete from esi_tokens where character_id = 2112000001"]),
    ("admin_requests", 1, ADMIN, "/admin/standing-requests", []),
    ("admin_requests_done", 1, ADMIN, "/admin/standing-requests?status=done", []),
    ("admin_requests_rejected", 1, ADMIN, "/admin/standing-requests?status=rejected", []),
    ("admin_requests_unknown_status", 1, ADMIN, "/admin/standing-requests?status=maybe", []),
    ("admin_requests_page_2", 1, ADMIN, "/admin/standing-requests?page=2", [
        "insert into standing_requests (id, subject_type, subject_id, requested_by_character_id, status, created_at, updated_at) "
        "select 1000 + n, 'character', 96000000 + n, 95000010, 'pending', {minutes:-500}, {minutes:-500} from ("
        "select a.n + b.n * 10 as n from (select 0 n union select 1 union select 2 union select 3 union select 4 "
        "union select 5 union select 6 union select 7 union select 8 union select 9) a, (select 0 n union select 1 "
        "union select 2) b) numbers where n < 28",
        "update standing_requests set created_at = date_sub(created_at, interval subject_id - 96000000 second) where subject_id >= 96000000",
    ]),
    ("admin_pilots", 1, ADMIN, "/admin/pilots", []),
    ("admin_pilots_search", 1, ADMIN, "/admin/pilots?search=%20Pilot%20", []),
    ("admin_pilots_unknown_view", 1, ADMIN, "/admin/pilots?view=maybe", []),
    ("admin_pilots_corporations", 1, ADMIN, "/admin/pilots?view=corporations", []),
    ("admin_pilots_corporations_search", 1, ADMIN, "/admin/pilots?view=corporations&search=kion", []),
    ("admin_pilots_corporations_member", 1, ADMIN, "/admin/pilots?view=corporations&search=tina", []),
    ("admin_pilots_alliances", 1, ADMIN, "/admin/pilots?view=alliances", []),
    ("admin_pilots_alliances_ticker", 1, ADMIN, "/admin/pilots?view=alliances&search=blua", []),
    ("admin_pilots_corporations_page_2", 1, ADMIN, "/admin/pilots?view=corporations&search=Corp&page=2", [
        "insert into corporations (id, name, ticker, created_at, updated_at) "
        "select 97000000 + n, concat('Filler Corp ', lpad(n, 2, '0')), 'FILL', {minutes:0}, {minutes:0} from ("
        "select a.n + b.n * 10 as n from (select 0 n union select 1 union select 2 union select 3 union select 4 "
        "union select 5 union select 6 union select 7 union select 8 union select 9) a, (select 0 n union select 1 "
        "union select 2) b) numbers where n < 22",
        "insert into characters (id, name, corporation_id, user_id, should_sync, created_at, updated_at) "
        "select 96500000 + n, concat('Filler Pilot ', lpad(n, 2, '0')), 97000000 + n, 4, 0, {minutes:0}, {minutes:0} "
        "from (select id - 97000000 as n from corporations where id between 97000000 and 97000099) numbers",
    ]),
    ("admin_settings", 1, ADMIN, "/admin/settings", []),
    ("admin_settings_alt", 1, ADMIN_ALT, "/admin/settings", []),
    ("admin_settings_no_source", 1, ADMIN, "/admin/settings", ["delete from standings_sources", "delete from discord_settings"]),
]

# (name, user, active character, method, path, body, page the form is on, extra SQL)
MUTATIONS = [
    ("sync_on", 2, NICOLAS, "PUT", f"/standings/{EMILE}", {"should_sync": True}, "/dashboard", []),
    ("sync_off", 2, NICOLAS, "PUT", f"/standings/{NICOLAS}", {"should_sync": False}, "/dashboard", []),
    ("sync_toggle_string", 2, NICOLAS, "PUT", f"/standings/{EMILE}", {"should_sync": "1"}, "/dashboard", []),
    ("sync_toggle_invalid", 2, NICOLAS, "PUT", f"/standings/{EMILE}", {"should_sync": "maybe"}, "/dashboard", []),
    ("sync_toggle_missing", 2, NICOLAS, "PUT", f"/standings/{EMILE}", {}, "/dashboard", []),
    ("sync_toggle_foreign", 2, NICOLAS, "PUT", f"/standings/{RED}", {"should_sync": True}, "/dashboard", []),
    ("sync_toggle_unknown", 2, NICOLAS, "PUT", "/standings/1", {"should_sync": True}, "/dashboard", []),
    ("sync_now", 2, NICOLAS, "POST", "/standings/sync", {}, "/dashboard", []),
    ("sync_now_nothing", 3, FRESH, "POST", "/standings/sync", {}, "/dashboard", []),
    ("sync_now_covered", 5, MEMBER, "POST", "/standings/sync", {}, "/dashboard", []),
    ("request_character", 4, RED, "POST", f"/standings/{RED}/request", {"type": "character"}, "/dashboard", []),
    ("request_corporation", 4, RED, "POST", f"/standings/{RED}/request", {"type": "corporation"}, "/dashboard", [
        "delete from source_contacts where contact_id = 98000004",
    ]),
    ("request_again_rejected", 4, RED, "POST", f"/standings/{RED}/request", {"type": "alliance"}, "/dashboard", [
        "delete from source_contacts where contact_id = 99000003",
    ]),
    ("request_pending", 3, FRESH, "POST", f"/standings/{FRESH}/request", {"type": "character"}, "/dashboard", []),
    ("request_done", 2, NICOLAS, "POST", f"/standings/{NICOLAS}/request", {"type": "character"}, "/dashboard", []),
    ("request_has_standing", 2, NICOLAS, "POST", f"/standings/{TINA}/request", {"type": "character"}, "/dashboard", []),
    ("request_no_alliance", 3, FRESH, "POST", f"/standings/{FRESH}/request", {"type": "alliance"}, "/dashboard", []),
    ("request_source_entity", 5, MEMBER, "POST", f"/standings/{MEMBER}/request", {"type": "alliance"}, "/dashboard", []),
    ("request_invalid_type", 4, RED, "POST", f"/standings/{RED}/request", {"type": "faction"}, "/dashboard", []),
    ("request_foreign", 4, RED, "POST", f"/standings/{FRESH}/request", {"type": "character"}, "/dashboard", []),
    ("request_no_source", 4, RED, "POST", f"/standings/{RED}/request", {"type": "character"}, "/dashboard", ["delete from standings_sources"]),
    ("switch_character", 2, NICOLAS, "PUT", f"/auth/character/{TINA}", {}, "/dashboard", []),
    ("switch_foreign", 2, NICOLAS, "PUT", f"/auth/character/{RED}", {}, "/dashboard", []),
    ("remove_alt", 2, NICOLAS, "DELETE", f"/auth/character/{EMILE}", {}, "/dashboard", []),
    ("remove_active_main", 2, NICOLAS, "DELETE", f"/auth/character/{NICOLAS}", {}, "/dashboard", []),
    ("remove_last", 3, FRESH, "DELETE", f"/auth/character/{FRESH}", {}, "/dashboard", []),
    ("remove_foreign", 2, NICOLAS, "DELETE", f"/auth/character/{RED}", {}, "/dashboard", []),
    ("set_main", 4, RED, "PUT", f"/auth/character/{RED}/main", {}, "/dashboard", []),
    ("set_main_completes_setup", 3, FRESH, "PUT", f"/auth/character/{FRESH}/main", {}, "/dashboard", []),
    ("set_main_foreign", 4, RED, "PUT", f"/auth/character/{FRESH}/main", {}, "/dashboard", []),
    ("setup_complete", 3, FRESH, "POST", "/setup/complete", {}, "/dashboard", []),
    ("logout", 2, NICOLAS, "POST", "/logout", {}, "/dashboard", []),
    ("settings_put", 2, NICOLAS, "PUT", "/settings", {}, "/settings/appearance", []),
    ("admin_source_corporation", 1, ADMIN, "PUT", "/admin/standings", {"type": "corporation"}, "/admin/settings", []),
    ("admin_source_same", 1, ADMIN, "PUT", "/admin/standings", {"type": "alliance"}, "/admin/settings", []),
    ("admin_source_character", 1, ADMIN, "PUT", "/admin/standings", {"type": "character"}, "/admin/settings", []),
    ("admin_source_invalid", 1, ADMIN, "PUT", "/admin/standings", {"type": "faction"}, "/admin/settings", []),
    ("admin_source_missing", 1, ADMIN, "PUT", "/admin/standings", {}, "/admin/settings", []),
    ("admin_source_no_alliance", 1, ADMIN_ALT, "PUT", "/admin/standings", {"type": "alliance"}, "/admin/settings", []),
    ("admin_source_forbidden", 2, NICOLAS, "PUT", "/admin/standings", {"type": "corporation"}, "/dashboard", []),
    ("admin_resolve_done", 1, ADMIN, "PUT", "/admin/standing-requests/1", {"status": "done"}, "/admin/standing-requests", []),
    ("admin_resolve_rejected", 1, ADMIN, "PUT", "/admin/standing-requests/2", {"status": "rejected"}, "/admin/standing-requests", []),
    ("admin_resolve_invalid", 1, ADMIN, "PUT", "/admin/standing-requests/1", {"status": "maybe"}, "/admin/standing-requests", []),
    ("admin_resolve_unknown", 1, ADMIN, "PUT", "/admin/standing-requests/999", {"status": "done"}, "/admin/standing-requests", []),
    ("admin_resolve_forbidden", 2, NICOLAS, "PUT", "/admin/standing-requests/1", {"status": "done"}, "/dashboard", []),
    ("admin_discord", 1, ADMIN, "PUT", "/admin/discord-settings", {"webhook_url": "https://discord.com/api/webhooks/1/abc", "role_id": "123456"}, "/admin/settings", []),
    ("admin_discord_clear", 1, ADMIN, "PUT", "/admin/discord-settings", {"webhook_url": "", "role_id": None}, "/admin/settings", []),
    ("admin_discord_first", 1, ADMIN, "PUT", "/admin/discord-settings", {"webhook_url": "https://discord.com/api/webhooks/2/x"}, "/admin/settings", ["delete from discord_settings"]),
    ("admin_discord_invalid", 1, ADMIN, "PUT", "/admin/discord-settings", {"webhook_url": "not-a-url", "role_id": "abc"}, "/admin/settings", []),
    ("admin_discord_http", 1, ADMIN, "PUT", "/admin/discord-settings", {"webhook_url": "http://discord.com/api/webhooks/1/abc", "role_id": "12 3"}, "/admin/settings", []),
    ("admin_discord_number", 1, ADMIN, "PUT", "/admin/discord-settings", {"webhook_url": None, "role_id": 123}, "/admin/settings", []),
    ("admin_discord_forbidden", 2, NICOLAS, "PUT", "/admin/discord-settings", {"webhook_url": "https://discord.com/x"}, "/dashboard", []),
    ("admin_sync_now", 1, ADMIN, "POST", "/admin/standings/sync", {}, "/admin/settings", []),
]

MUTATION_FAKES = {"admin_sync_now": source_fakes(SOURCE_CHANGED)}


def record_pages():
    for name, user, character, path, extra in PAGES:
        seed(extra)
        write_fakes({})
        browser = Browser()
        if user:
            browser.login(user, character)
        status, headers, body = browser.request("GET", path, headers={"Accept": "text/html"})
        out = {"user": user, "character": character, "path": path, "sql": extra, "status": status}
        if 300 <= status < 400:
            out["location"] = headers.get("Location")
        else:
            page = page_object(body)
            if page:
                out["component"] = page["component"]
                out["props"] = page["props"]
            else:
                out["html"] = body
        save("pages", name, out)


def follow(browser, location):
    """The page a redirect lands on: its flash and errors."""
    path = site_path(location or "")
    if path is None:
        return None
    status, headers, body = browser.request("GET", path, headers={"Accept": "text/html"})
    if 300 <= status < 400:
        return {"path": path, "status": status, "location": headers.get("Location")}
    page = page_object(body)
    if not page:
        return {"path": path, "status": status}
    return {
        "path": path,
        "status": status,
        "flash": page["props"].get("flash"),
        "errors": page["props"].get("errors"),
        "auth_user": page["props"]["auth"]["user"],
        "active_character": (page["props"]["auth"].get("active_character") or {}).get("id"),
    }


def record_mutations():
    for name, user, character, method, path, body, on_page, extra in MUTATIONS:
        seed(extra)
        fakes = MUTATION_FAKES.get(name, {})
        write_fakes(fakes)
        browser = Browser()
        browser.login(user, character)
        browser.request("GET", on_page, headers={"Accept": "text/html"})
        status, headers, answer = browser.request(method, path, body=body, headers={
            "Accept": "text/html",
            "Content-Type": "application/json",
            "X-XSRF-TOKEN": browser.xsrf(),
            "Referer": BASE + on_page,
        })
        out = {
            "user": user, "character": character, "method": method, "path": path, "body": body,
            "page": on_page, "sql": extra, "fakes": fakes, "status": status,
        }
        if 300 <= status < 400:
            out["location"] = headers.get("Location")
            out["after"] = follow(browser, out["location"])
        elif status != 204:
            out["html"] = answer
        out["jobs"] = jobs()
        out["http"] = http_log()
        out["rows"] = rows()
        save("mutations", name, out)


def guest_logout():
    seed()
    write_fakes({})
    browser = Browser()
    browser.request("GET", "/", headers={"Accept": "text/html"})
    status, _, _ = browser.request("POST", "/logout", body={}, headers={
        "Accept": "text/html", "Content-Type": "application/json", "X-XSRF-TOKEN": browser.xsrf(),
    })
    save("mutations", "logout_guest", {"method": "POST", "path": "/logout", "status": status})


def background(name, fakes, steps, extra=()):
    seed(extra)
    write_fakes(fakes)
    out = {"sql": list(extra), "fakes": fakes, "steps": []}
    for step in steps:
        if step[0] == "artisan":
            code, output = artisan(*step[1:])
            out["steps"].append({"artisan": list(step[1:]), "exit": code, "jobs": jobs()})
        elif step[0] == "queue":
            artisan("queue:work", "--stop-when-empty")
            out["steps"].append({"queue": True, "jobs": jobs(), "failed": failed_jobs()})
        elif step[0] == "php":
            php(step[1])
            out["steps"].append({"php": step[1], "jobs": jobs()})
    out["http"] = http_log()
    out["rows"] = rows()
    save("background", name, out)


def record_background():
    discord = {f"POST {WEBHOOK}": {"status": 204}}
    background("sync_changed", {**source_fakes(SOURCE_CHANGED)}, [("artisan", "standings:sync")])
    background("sync_unchanged", {**source_fakes(SOURCE_UNCHANGED)}, [("artisan", "standings:sync")])
    background("sync_unchanged_forced", {**source_fakes(SOURCE_UNCHANGED)}, [("artisan", "standings:sync", "--force")])
    background("sync_no_source", {}, [("artisan", "standings:sync")], ["delete from standings_sources"])
    background(
        "sync_unreadable",
        discord,
        [("artisan", "standings:sync"), ("artisan", "standings:sync"), ("queue",)],
        ["delete from esi_tokens where character_id = 2112000001"],
    )
    background(
        "sync_unreadable_corporation",
        discord,
        [("artisan", "standings:sync"), ("queue",)],
        [
            "update standings_sources set type = 'corporation', entity_id = 98000001",
            "update discord_settings set role_id = null",
        ],
    )
    background(
        "sync_forbidden",
        {f"GET {ESI}/alliances/99000002/contacts/": {"status": 403, "body": {"error": "forbidden"}}},
        [("artisan", "standings:sync")],
    )
    background(
        "character_sync",
        {**NICOLAS_CONTACTS},
        [("php", f"App\\Jobs\\SyncCharacterStandings::dispatchSync(App\\Models\\Character::find({NICOLAS}));")],
    )
    background(
        "character_sync_unreadable",
        {},
        [("php", f"App\\Jobs\\SyncCharacterStandings::dispatchSync(App\\Models\\Character::find({ADMIN_ALT}));")],
    )
    background(
        "character_sync_refresh",
        {
            **NICOLAS_CONTACTS,
            f"POST {SSO}/v2/oauth/token": {"body": {
                "access_token": "new-access-token", "expires_in": 1199, "token_type": "Bearer",
                "refresh_token": "new-refresh-token",
            }},
        },
        [("php", f"App\\Jobs\\SyncCharacterStandings::dispatchSync(App\\Models\\Character::find({NICOLAS}));")],
        ["update esi_tokens set expires_at = {minutes:2} where character_id = 95000001"],
    )
    background(
        "character_sync_write_only",
        {},
        [("php", f"App\\Jobs\\SyncCharacterStandings::dispatchSync(App\\Models\\Character::find({RED}));")],
    )
    background(
        "affiliations",
        {
            f"POST {ESI}/characters/affiliation/": {"body": [
                {"character_id": ADMIN, "corporation_id": 98000001, "alliance_id": 99000002},
                {"character_id": ADMIN_ALT, "corporation_id": 98000003},
                {"character_id": NICOLAS, "corporation_id": 98000002, "alliance_id": 99000001},
                {"character_id": TINA, "corporation_id": 98000002, "alliance_id": 99000001},
                {"character_id": EMILE, "corporation_id": 98000005},
                {"character_id": FRESH, "corporation_id": 98000008, "alliance_id": 99000005},
                {"character_id": RED, "corporation_id": 98000004, "alliance_id": 99000003},
                {"character_id": MEMBER, "corporation_id": 98000001, "alliance_id": 99000002},
                {"character_id": 95000100, "corporation_id": 98000002, "alliance_id": 99000001},
                {"character_id": 95000101, "corporation_id": 98000004, "alliance_id": 99000003},
            ]},
            f"GET {ESI}/corporations/98000002/": {"body": corporation_payload("Blue Corp", "BLUC", 99000001)},
            f"GET {ESI}/corporations/98000099/": {"body": corporation_payload("Ghost Corp", "GHST")},
            f"GET {ESI}/corporations/98000003/": {"body": corporation_payload("Kion Trading Inc.", "KION", 99000001)},
            f"GET {ESI}/corporations/98000004/": {"body": corporation_payload("Red Corp Renamed", "REDC", 99000003)},
            f"GET {ESI}/corporations/98000008/": {"body": corporation_payload("Eighth Corp", "EIGHT", 99000005)},
            f"GET {ESI}/alliances/99000005/": {"body": alliance_payload("Fifth Alliance", "FIFTH")},
        },
        [("artisan", "characters:sync-affiliations")],
    )
    background(
        "check_tokens",
        {f"POST {ESI}/characters/{ADMIN}/mail/": {"status": 201, "body": "12345"}},
        [("artisan", "standings:check-tokens"), ("artisan", "standings:check-tokens"), ("queue",)],
    )
    background(
        "check_tokens_no_mail_scope",
        {},
        [("artisan", "standings:check-tokens")],
        [
            "delete from esi_token_scope where esi_token_id = 1 and esi_scope_id = "
            "(select id from esi_scopes where name = 'esi-mail.send_mail.v1')",
        ],
    )
    background(
        "notify_request",
        discord,
        [("php", "App\\Jobs\\NotifyStandingRequest::dispatch(App\\Models\\StandingRequest::find(2));"), ("queue",)],
    )
    background(
        "notify_request_alliance",
        discord,
        [("php", "App\\Jobs\\NotifyStandingRequest::dispatch(App\\Models\\StandingRequest::find(3));"), ("queue",)],
        ["update discord_settings set role_id = null"],
    )
    background(
        "notify_request_env_webhook",
        {},
        [("php", "App\\Jobs\\NotifyStandingRequest::dispatch(App\\Models\\StandingRequest::find(1));"), ("queue",)],
        ["delete from discord_settings"],
    )


def main():
    if sys.argv[1:] == ["seed"]:
        seed()
        return
    sections = sys.argv[1:] or ["pages", "mutations", "background"]
    os.makedirs(OUT, exist_ok=True)
    if "pages" in sections:
        # Only the pages show times of the dataset; the other sections
        # compare rows without them.
        with open(os.path.join(OUT, "meta.json"), "w", encoding="utf-8") as file:
            json.dump({"now": NOW.strftime("%Y-%m-%dT%H:%M:%SZ")}, file)
            file.write("\n")
        record_pages()
    if "mutations" in sections:
        record_mutations()
        guest_logout()
    if "background" in sections:
        record_background()


if __name__ == "__main__":
    main()

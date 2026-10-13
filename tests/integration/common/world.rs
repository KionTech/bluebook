//! A small world the page, form and background suites start from: an
//! admin with an alt, pilots in blue, neutral and red entities, a member
//! of the source alliance, the source with its standings, a ledger and a
//! few standing requests.
//!
//! Users: 1 the admin (Admin Pilot, alt "admin alt"), 2 Nicolas Pilot
//! (alts Tina Pilot and Émile Pilot), 3 Fresh Pilot (not set up), 4 Red
//! Pilot (no main), 5 Member Pilot (in the source alliance).

use time::Duration;

use super::TestApp;

pub const ADMIN: i64 = 2_112_000_001;
pub const ADMIN_ALT: i64 = 2_112_000_002;
pub const NICOLAS: i64 = 95_000_001;
pub const TINA: i64 = 95_000_002;
pub const EMILE: i64 = 95_000_003;
pub const FRESH: i64 = 95_000_010;
pub const RED: i64 = 95_000_020;
pub const MEMBER: i64 = 95_000_030;

const ALLIANCES: [(i64, &str, &str); 3] = [
    (99_000_001, "Blue Alliance", "BLUA"),
    (99_000_002, "Home Alliance", "HOME"),
    (99_000_003, "Red Alliance", "RED"),
];

const CORPORATIONS: [(i64, &str, &str, Option<i64>); 5] = [
    (98_000_001, "Home Corp", "HOMC", Some(99_000_002)),
    (98_000_002, "Blue Corp", "BLUC", Some(99_000_001)),
    (98_000_003, "Kion Trading Inc.", "KION", None),
    (98_000_004, "Red Corp", "REDC", Some(99_000_003)),
    (98_000_005, "Écho Industries", "ECHO", None),
];

/// id, name, main character, setup completed (minutes from now).
const USERS: [(i64, &str, Option<i64>, Option<i64>); 5] = [
    (1, "Admin Pilot", Some(ADMIN), Some(-1440)),
    (2, "Nicolas Pilot", Some(NICOLAS), Some(-1440)),
    (3, "Fresh Pilot", None, None),
    (4, "Red Pilot", None, Some(-600)),
    (5, "Member Pilot", Some(MEMBER), Some(-600)),
];

/// id, name, corporation, alliance, user, should sync.
#[allow(clippy::type_complexity)]
const CHARACTERS: [(
    i64,
    Option<&str>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    bool,
); 11] = [
    (
        ADMIN,
        Some("Admin Pilot"),
        Some(98_000_001),
        Some(99_000_002),
        Some(1),
        false,
    ),
    (
        ADMIN_ALT,
        Some("admin alt"),
        Some(98_000_003),
        None,
        Some(1),
        true,
    ),
    (
        NICOLAS,
        Some("Nicolas Pilot"),
        Some(98_000_002),
        Some(99_000_001),
        Some(2),
        true,
    ),
    (
        TINA,
        Some("Tina Pilot"),
        Some(98_000_003),
        None,
        Some(2),
        true,
    ),
    (
        EMILE,
        Some("Émile Pilot"),
        Some(98_000_005),
        None,
        Some(2),
        false,
    ),
    (
        FRESH,
        Some("Fresh Pilot"),
        Some(98_000_003),
        None,
        Some(3),
        false,
    ),
    (
        RED,
        Some("Red Pilot"),
        Some(98_000_004),
        Some(99_000_003),
        Some(4),
        true,
    ),
    (
        MEMBER,
        Some("Member Pilot"),
        Some(98_000_001),
        Some(99_000_002),
        Some(5),
        true,
    ),
    (
        95_000_100,
        Some("Contact One"),
        Some(98_000_002),
        Some(99_000_001),
        None,
        false,
    ),
    (
        95_000_101,
        Some("Spy Pilot"),
        Some(98_000_004),
        Some(99_000_003),
        None,
        false,
    ),
    (3_018_996, None, None, None, None, false),
];

const ALL_SCOPES: &[&str] = &[
    "publicData",
    "esi-characters.read_contacts.v1",
    "esi-characters.write_contacts.v1",
    "esi-corporations.read_contacts.v1",
    "esi-alliances.read_contacts.v1",
    "esi-mail.send_mail.v1",
];
const CONTACT_SCOPES: &[&str] = &[
    "esi-characters.read_contacts.v1",
    "esi-characters.write_contacts.v1",
];
const WRITE_SCOPE: &[&str] = &["esi-characters.write_contacts.v1"];

/// Tokens 1 to 6, each valid for another hour: `access-token-N`,
/// `refresh-token-N`.
const TOKENS: [(i64, &[&str]); 6] = [
    (ADMIN, ALL_SCOPES),
    (ADMIN_ALT, CONTACT_SCOPES),
    (NICOLAS, CONTACT_SCOPES),
    (EMILE, &[]),
    (RED, WRITE_SCOPE),
    (MEMBER, WRITE_SCOPE),
];

/// The source's standings, Home Alliance's contacts.
pub const SOURCE_CONTACTS: [(i64, &str, f64); 10] = [
    (99_000_001, "alliance", 10.0),
    (98_000_002, "corporation", 10.0),
    (95_000_100, "character", 5.0),
    (98_000_099, "corporation", 5.0),
    (95_000_002, "character", 0.0),
    (98_000_003, "corporation", -5.0),
    (500_001, "faction", -5.0),
    (99_000_003, "alliance", -10.0),
    (95_000_101, "character", -10.0),
    (98_000_004, "corporation", -10.0),
];

/// Nicolas Pilot's ledger.
const SYNCED: [(i64, i64, f64); 3] = [
    (NICOLAS, 99_000_001, 10.0),
    (NICOLAS, 98_000_002, 10.0),
    (NICOLAS, 12_345, 5.0),
];

/// id, subject type, subject, requested by, status, created (minutes from
/// now).
const REQUESTS: [(i64, &str, i64, i64, &str, i64); 5] = [
    (1, "character", FRESH, FRESH, "pending", -60),
    (2, "corporation", 98_000_003, TINA, "pending", -30),
    (3, "alliance", 99_000_003, RED, "rejected", -120),
    (4, "character", NICOLAS, NICOLAS, "done", -240),
    (5, "corporation", 98_000_005, EMILE, "pending", -45),
];

impl TestApp {
    /// An app at `DEFAULT_NOW` with the world seeded and Admin Pilot as
    /// the admin character.
    pub async fn world() -> Self {
        let mut app = Self::new().await;
        app.admins(&[ADMIN]);
        app.seed_world().await;
        app
    }

    pub async fn seed_world(&self) {
        let now = self.clock.now();
        let at = |minutes: i64| now + Duration::minutes(minutes);
        let pool = &self.pool;
        let run = |sql: &str| sqlx::query(sqlx::AssertSqlSafe(sql.to_owned()));

        for (id, name, ticker) in ALLIANCES {
            run(
                "insert into alliances (id, name, ticker, created_at, updated_at)
                 values ($1, $2, $3, $4, $4)",
            )
            .bind(id)
            .bind(name)
            .bind(ticker)
            .bind(now)
            .execute(pool)
            .await
            .expect("an alliance");
        }
        for (id, name, ticker, alliance) in CORPORATIONS {
            run(
                "insert into corporations (id, name, ticker, alliance_id, created_at, updated_at)
                 values ($1, $2, $3, $4, $5, $5)",
            )
            .bind(id)
            .bind(name)
            .bind(ticker)
            .bind(alliance)
            .bind(now)
            .execute(pool)
            .await
            .expect("a corporation");
        }
        run("insert into factions (id, name, created_at, updated_at)
             values (500001, 'Caldari State', $1, $1)")
        .bind(now)
        .execute(pool)
        .await
        .expect("a faction");
        for (id, name, _, setup) in USERS {
            run(
                "insert into users (id, name, setup_completed_at, created_at, updated_at)
                 values ($1, $2, $3, $4, $4)",
            )
            .bind(id)
            .bind(name)
            .bind(setup.map(at))
            .bind(now)
            .execute(pool)
            .await
            .expect("a user");
        }
        for (number, (id, name, corporation, alliance, user, should_sync)) in
            CHARACTERS.into_iter().enumerate()
        {
            run(
                "insert into characters (id, name, corporation_id, alliance_id, user_id,
                    character_owner_hash, should_sync, created_at, updated_at)
                 values ($1, $2, $3, $4, $5, $6, $7, $8, $8)",
            )
            .bind(id)
            .bind(name)
            .bind(corporation)
            .bind(alliance)
            .bind(user)
            .bind(user.map(|_| format!("owner-hash-{}", number + 1)))
            .bind(should_sync)
            .bind(now)
            .execute(pool)
            .await
            .expect("a character");
        }
        for (id, _, main, _) in USERS {
            run("update users set main_character_id = $2 where id = $1")
                .bind(id)
                .bind(main)
                .execute(pool)
                .await
                .expect("a main character");
        }
        for (index, (character, scopes)) in TOKENS.into_iter().enumerate() {
            let number = index as i64 + 1;
            run(
                "insert into esi_tokens (id, character_id, access_token, token_type,
                    refresh_token, character_owner_hash, scopes, expires_at, created_at,
                    updated_at)
                 values ($1, $2, $3, 'Bearer', $4, $5, $6, $7, $8, $8)",
            )
            .bind(number)
            .bind(character)
            .bind(format!("access-token-{number}"))
            .bind(format!("refresh-token-{number}"))
            .bind(format!("owner-hash-{number}"))
            .bind(
                scopes
                    .iter()
                    .map(|scope| (*scope).to_owned())
                    .collect::<Vec<_>>(),
            )
            .bind(at(60))
            .bind(now)
            .execute(pool)
            .await
            .expect("a token");
        }
        run(
            "insert into standings_sources (type, entity_id, last_synced_at, created_at,
                updated_at)
             values ('alliance', 99000002, $1, $2, $2)",
        )
        .bind(at(-2))
        .bind(now)
        .execute(pool)
        .await
        .expect("a source");
        for (contact, kind, standing) in SOURCE_CONTACTS {
            run(
                "insert into source_contacts (contact_id, contact_type, standing, created_at,
                    updated_at)
                 values ($1, $2, $3::float8::numeric(4, 2), $4, $4)",
            )
            .bind(contact)
            .bind(kind)
            .bind(standing)
            .bind(now)
            .execute(pool)
            .await
            .expect("a source contact");
        }
        for (character, contact, standing) in SYNCED {
            run(
                "insert into character_synced_contacts (character_id, contact_id, standing,
                    created_at, updated_at)
                 values ($1, $2, $3::float8::numeric(4, 2), $4, $4)",
            )
            .bind(character)
            .bind(contact)
            .bind(standing)
            .bind(now)
            .execute(pool)
            .await
            .expect("a synced contact");
        }
        for (id, kind, subject, by, status, created) in REQUESTS {
            run(
                "insert into standing_requests (id, subject_type, subject_id,
                    requested_by_character_id, status, created_at, updated_at)
                 values ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(id)
            .bind(kind)
            .bind(subject)
            .bind(by)
            .bind(status)
            .bind(at(created))
            .bind(now)
            .execute(pool)
            .await
            .expect("a standing request");
        }
        run(
            "insert into discord_settings (webhook_url, role_id, created_at, updated_at)
             values ($1, '777', $2, $2)",
        )
        .bind(self.discord.webhook())
        .bind(now)
        .execute(pool)
        .await
        .expect("discord settings");
        for table in ["users", "esi_tokens", "standing_requests"] {
            run(&format!(
                "select setval(pg_get_serial_sequence('{table}', 'id'), (select max(id) from {table}))"
            ))
            .execute(pool)
            .await
            .expect("a sequence");
        }
    }

    /// Runs a statement against the test database, for a scenario's
    /// change to the world.
    pub async fn exec(&self, sql: &str) {
        sqlx::query(sqlx::AssertSqlSafe(sql.to_owned()))
            .execute(&self.pool)
            .await
            .unwrap_or_else(|error| panic!("{sql}: {error}"));
    }
}

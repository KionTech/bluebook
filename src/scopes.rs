//! The ESI scopes: every scope the login may ask for (the scope catalogue
//! of `nicolaskion/eve`), the ones the app uses, and the grant links that
//! ask EVE for them.

pub const PUBLIC_DATA: &str = "publicData";
pub const READ_CHARACTER_CONTACTS: &str = "esi-characters.read_contacts.v1";
pub const WRITE_CHARACTER_CONTACTS: &str = "esi-characters.write_contacts.v1";
pub const READ_CORPORATION_CONTACTS: &str = "esi-corporations.read_contacts.v1";
pub const READ_ALLIANCE_CONTACTS: &str = "esi-alliances.read_contacts.v1";
pub const SEND_MAIL: &str = "esi-mail.send_mail.v1";

/// What a pilot grants to sync standings into its contacts.
pub const SYNC_SCOPES: [&str; 2] = [READ_CHARACTER_CONTACTS, WRITE_CHARACTER_CONTACTS];

/// What the admin's source character grants: reading every kind of
/// source, and mail for the lapsed token reminders.
pub const ADMIN_SCOPES: [&str; 6] = [
    PUBLIC_DATA,
    READ_CHARACTER_CONTACTS,
    WRITE_CHARACTER_CONTACTS,
    READ_CORPORATION_CONTACTS,
    READ_ALLIANCE_CONTACTS,
    SEND_MAIL,
];

/// Every scope a login may ask for; a token stores only these.
pub const ALL: [&str; 71] = [
    "esi-assets.read_assets.v1",
    "esi-markets.structure_markets.v1",
    "publicData",
    "esi-ui.open_window.v1",
    "esi-contracts.read_character_contracts.v1",
    "esi-contracts.read_corporation_contracts.v1",
    "esi-killmails.read_killmails.v1",
    "esi-killmails.read_corporation_killmails.v1",
    "esi-mail.read_mail.v1",
    "esi-mail.send_mail.v1",
    "esi-mail.organize_mail.v1",
    "esi-universe.read_structures.v1",
    "esi-assets.read_corporation_assets.v1",
    "esi-wallet.read_character_wallet.v1",
    "esi-location.read_location.v1",
    "esi-location.read_online.v1",
    "esi-location.read_ship_type.v1",
    "esi-ui.write_waypoint.v1",
    "esi-corporations.read_divisions.v1",
    "esi-corporations.read_structures.v1",
    "esi-characters.read_contacts.v1",
    "esi-characters.write_contacts.v1",
    "esi-corporations.read_contacts.v1",
    "esi-alliances.read_contacts.v1",
    "esi-corporations.read_blueprints.v1",
    "esi-corporations.read_container_logs.v1",
    "esi-corporations.read_facilities.v1",
    "esi-corporations.read_medals.v1",
    "esi-corporations.read_corporation_membership.v1",
    "esi-corporations.track_members.v1",
    "esi-corporations.read_titles.v1",
    "esi-wallet.read_corporation_wallets.v1",
    "esi-corporations.read_standings.v1",
    "esi-corporations.read_starbases.v1",
    "esi-characters.read_agents_research.v1",
    "esi-characters.read_blueprints.v1",
    "esi-characters.read_fatigue.v1",
    "esi-characters.read_medals.v1",
    "esi-characters.read_notifications.v1",
    "esi-characters.read_corporation_roles.v1",
    "esi-characters.read_standings.v1",
    "esi-characters.read_titles.v1",
    "esi-characters.read_fw_stats.v1",
    "esi-corporations.read_fw_stats.v1",
    "esi-skills.read_skills.v1",
    "esi-skills.read_skillqueue.v1",
    "esi-clones.read_clones.v1",
    "esi-clones.read_implants.v1",
    "esi-fleets.read_fleet.v1",
    "esi-fleets.write_fleet.v1",
    "esi-industry.read_character_jobs.v1",
    "esi-industry.read_character_mining.v1",
    "esi-industry.read_corporation_mining.v1",
    "esi-industry.read_corporation_jobs.v1",
    "esi-markets.read_character_orders.v1",
    "esi-markets.read_corporation_orders.v1",
    "esi-characters.read_loyalty.v1",
    "esi-calendar.read_calendar_events.v1",
    "esi-calendar.respond_calendar_events.v1",
    "esi-fittings.read_fittings.v1",
    "esi-fittings.write_fittings.v1",
    "esi-planets.manage_planets.v1",
    "esi-planets.read_customs_offices.v1",
    "esi-structures.read_character.v1",
    "esi-structures.read_corporation.v1",
    "esi-corporations.read_projects.v1",
    "esi-access.read_lists.v1",
    "esi-characters.read_freelance_jobs.v1",
    "esi-corporations.read_freelance_jobs.v1",
    "esi-activities.read_character.v1",
    "esi-search.search_structures.v1",
];

pub fn is_known(scope: &str) -> bool {
    ALL.contains(&scope)
}

/// The login link that adds a character to the signed-in account with
/// these scopes (`EveSso::grantScopesUrl`).
pub fn grant_url(app_url: &str, scopes: &[&str]) -> String {
    let joined = scopes.join(",");
    let encoded: String = form_urlencoded::byte_serialize(joined.as_bytes()).collect();
    format!("{app_url}/eve?add_to_account=1&scopes={encoded}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_the_grant_links_like_laravels_route_helper() {
        assert_eq!(
            grant_url("https://bluebook.nicolaskion.dev", &SYNC_SCOPES),
            "https://bluebook.nicolaskion.dev/eve?add_to_account=1&scopes=esi-characters.read_contacts.v1%2Cesi-characters.write_contacts.v1"
        );
    }

    #[test]
    fn knows_the_catalogue() {
        assert!(is_known("esi-mail.send_mail.v1"));
        assert!(!is_known("nope"));
        assert_eq!(ALL.len(), 71);
    }
}

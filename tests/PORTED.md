# Ported tests

Every test of the Laravel app (`tests/` on `main`, Pest, datasets counted per case) and the test that ports it: a Rust integration test (`suite::name` in `tests/integration`) or a Playwright test (`frontend/e2e`, file and name). Where the PHP test asserted a Laravel bug, the port asserts the fix and names its FIXES.md entry.

| PHP test | Description | Ported to |
| --- | --- | --- |
| tests/Browser/AdministrationBrowserTest.php | shows the overview with stats and the section tabs | `administration.spec.ts` "shows the overview with stats and the section tabs" |
| tests/Browser/AdministrationBrowserTest.php | shows the read-only source with confirm-to-change on the settings page | `administration.spec.ts` "shows the read-only source with confirm-to-change on the settings page" |
| tests/Browser/AdministrationBrowserTest.php | confirms and resolves a standing request with a toast | `administration.spec.ts` "confirms and resolves a standing request with a toast" |
| tests/Browser/AdministrationBrowserTest.php | shows a success toast after saving discord settings | `administration.spec.ts` "shows a success toast after saving discord settings" |
| tests/Browser/AdministrationBrowserTest.php | lists pilots with their main character and alts | `administration.spec.ts` "lists pilots with their main character and alts" |
| tests/Browser/PilotsBrowserTest.php | renders the pilots corporation view with standings and member characters | `administration.spec.ts` "renders the pilots corporation view with standings and member characters" |
| tests/Browser/PilotsBrowserTest.php | renders the flat character list with standings | `administration.spec.ts` "renders the flat character list with standings" |
| tests/Browser/SetupWizardBrowserTest.php | walks a new user through the wizard and sets the main character | `setup-wizard.spec.ts` "walks a new user through the wizard and sets the main character" |
| tests/Browser/SetupWizardBrowserTest.php | lets a new user skip the wizard | `setup-wizard.spec.ts` "lets a new user skip the wizard" |
| tests/Browser/SetupWizardBrowserTest.php | does not show the wizard to a user who completed setup | `setup-wizard.spec.ts` "does not show the wizard to a user who completed setup" |
| tests/Browser/StandingsBrowserTest.php | renders the welcome page without errors | `standings.spec.ts` "renders the welcome page without errors" |
| tests/Browser/StandingsBrowserTest.php | shows a grant-access banner for synced characters missing scopes | `standings.spec.ts` "shows a grant-access banner for synced characters missing scopes" |
| tests/Browser/StandingsBrowserTest.php | shows the standings hub with the user characters | `standings.spec.ts` "shows the standings hub with the user characters" |
| tests/Browser/StandingsBrowserTest.php | shows the source standings overview with resolved names | `standings.spec.ts` "shows the source standings overview with resolved names" |
| tests/Browser/StandingsBrowserTest.php | lists standings alphabetically within the same standing and type | `standings.spec.ts` "lists standings alphabetically within the same standing and type" |
| tests/Browser/StandingsBrowserTest.php | filters the standings list through the search bar | `standings.spec.ts` "filters the standings list through the search bar" |
| tests/Browser/StandingsBrowserTest.php | marks a character that inherits the source as inherited | `standings.spec.ts` "marks a character that inherits the source as inherited" |
| tests/Browser/StandingsBrowserTest.php | shows each character corporation and alliance | `standings.spec.ts` "shows each character corporation and alliance" |
| tests/Browser/StandingsBrowserTest.php | shows the live last-refreshed time and a next-refresh countdown | `standings.spec.ts` "shows the live last-refreshed time and a next-refresh countdown" |
| tests/Browser/StandingsBrowserTest.php | lets a user trigger a manual sync from the dashboard | `standings.spec.ts` "lets a user trigger a manual sync from the dashboard" |
| tests/Browser/StandingsBrowserTest.php | opens a request-standing dialog with the eligible entities | `standings.spec.ts` "opens a request-standing dialog with the eligible entities" |
| tests/Browser/StandingsBrowserTest.php | marks a request option that is already in the standings list | `standings.spec.ts` "marks a request option that is already in the standings list" |
| tests/Browser/StandingsBrowserTest.php | does not offer a request for a character that inherits the source | `standings.spec.ts` "does not offer a request for a character that inherits the source" |
| tests/Feature/AccessWarningsTest.php | flags opted-in characters missing the write-contacts token | `access_warnings::flags_opted_in_characters_missing_the_write_contacts_token` |
| tests/Feature/AccessWarningsTest.php | does not flag a scopeless character that never opted into syncing | `access_warnings::does_not_flag_a_scopeless_character_that_never_opted_into_syncing` |
| tests/Feature/AccessWarningsTest.php | does not flag a character that has the write-contacts token | `access_warnings::does_not_flag_a_character_that_has_the_write_contacts_token` |
| tests/Feature/AccessWarningsTest.php | flags an unreadable source for admins | `access_warnings::flags_an_unreadable_source_for_admins` |
| tests/Feature/AccessWarningsTest.php | does not flag the source for non-admins | `access_warnings::does_not_flag_the_source_for_non_admins` |
| tests/Feature/Admin/AdministrationTest.php | shows the overview with stats and recent requests to the admin | `admin::shows_the_overview_with_stats_and_recent_requests_to_the_admin` |
| tests/Feature/Admin/AdministrationTest.php | treats any of the comma-separated admin characters as an admin | `admin::treats_any_of_the_comma_separated_admin_characters_as_an_admin` |
| tests/Feature/Admin/AdministrationTest.php | forbids non-admins from the administration page | `admin::forbids_non_admins_from_the_administration_page` |
| tests/Feature/Admin/AdministrationTest.php | lets the admin mark a request as done | `admin::lets_the_admin_mark_a_request_as_done` |
| tests/Feature/Admin/AdministrationTest.php | lets the admin reject a request | `admin::lets_the_admin_reject_a_request` |
| tests/Feature/Admin/AdministrationTest.php | forbids non-admins from resolving requests | `admin::forbids_non_admins_from_resolving_requests` |
| tests/Feature/Admin/AdministrationTest.php | lets the admin update the discord settings | `admin::lets_the_admin_update_the_discord_settings` |
| tests/Feature/Admin/AdministrationTest.php | validates the discord settings | `admin::validates_the_discord_settings` |
| tests/Feature/Admin/AdministrationTest.php | forbids non-admins from updating discord settings | `admin::forbids_non_admins_from_updating_discord_settings` |
| tests/Feature/Admin/AdministrationTest.php | lists every registered character under its account on the pilots page | `admin::lists_every_registered_character_under_its_account_on_the_pilots_page` |
| tests/Feature/Admin/AdministrationTest.php | shows pending requests on the standing requests page by default | `admin::shows_pending_requests_on_the_standing_requests_page_by_default` |
| tests/Feature/Admin/AdministrationTest.php | shows done requests only when the done filter is selected | `admin::shows_done_requests_only_when_the_done_filter_is_selected` |
| tests/Feature/Admin/AdministrationTest.php | paginates the standing requests list | `admin::paginates_the_standing_requests_list` |
| tests/Feature/Admin/PilotsTest.php | lists every registered character with its account and standing | `admin_pilots::lists_every_registered_character_with_its_account_and_standing` |
| tests/Feature/Admin/PilotsTest.php | passes the search term through for client-side filtering | `admin_pilots::passes_the_search_term_through_for_client_side_filtering` |
| tests/Feature/Admin/PilotsTest.php | forbids non-admins from the pilots page | `admin_pilots::forbids_non_admins_from_the_pilots_page` |
| tests/Feature/Admin/PilotsTest.php | groups corporations with their standing and member characters | `admin_pilots::groups_corporations_with_their_standing_and_member_characters` |
| tests/Feature/Admin/PilotsTest.php | groups alliances with their member characters | `admin_pilots::groups_alliances_with_their_member_characters` |
| tests/Feature/Admin/PilotsTest.php | filters group views by character name on the server | `admin_pilots::filters_group_views_by_character_name_on_the_server` |
| tests/Feature/Admin/PilotsTest.php | marks characters and corporations covered by the source as green source standings | `admin_pilots::marks_characters_and_corporations_covered_by_the_source_as_green_source_standings` |
| tests/Feature/Admin/PilotsTest.php | excludes corporations without registered characters | `admin_pilots::excludes_corporations_without_registered_characters` |
| tests/Feature/Admin/SettingsPageTest.php | shows the settings page with source options and discord settings | `admin_settings::shows_the_settings_page_with_source_options_and_discord_settings` |
| tests/Feature/Admin/SettingsPageTest.php | forbids non-admins from the settings page | `admin_settings::forbids_non_admins_from_the_settings_page` |
| tests/Feature/Admin/StandingRequestDisplayTest.php | passes the inherited effective standing for each request to the admin console | `admin::passes_the_inherited_effective_standing_for_each_request_to_the_admin_console` |
| tests/Feature/Admin/StandingsSourceTest.php | forbids non-admins from updating the source | `admin_source::forbids_non_admins_from_updating_the_source` |
| tests/Feature/Admin/StandingsSourceTest.php | derives the entity id from the admin character | `admin_source::derives_the_entity_id_from_the_admin_character` |
| tests/Feature/Admin/StandingsSourceTest.php | uses the alliance id when the alliance is selected | `admin_source::uses_the_alliance_id_when_the_alliance_is_selected` |
| tests/Feature/Admin/StandingsSourceTest.php | rejects a source type the admin character does not have | `admin_source::rejects_a_source_type_the_admin_character_does_not_have` |
| tests/Feature/Admin/StandingsSourceTest.php | clears canonical contacts when the source changes | `admin_source::clears_canonical_contacts_when_the_source_changes` |
| tests/Feature/Admin/StandingsSourceTest.php | validates the source type | `admin_source::validates_the_source_type` |
| tests/Feature/Admin/StandingsSourceTest.php | queues a sync for opted-in characters when the admin clicks sync now | `admin_source::queues_a_sync_for_opted_in_characters_when_the_admin_clicks_sync_now` |
| tests/Feature/Auth/AffiliationSyncTest.php | fetches and stores corporation and alliance details on login | `login::fetches_and_stores_corporation_and_alliance_details_on_login` |
| tests/Feature/Auth/AffiliationSyncTest.php | stores a character with no alliance | `login::stores_a_character_with_no_alliance` |
| tests/Feature/Auth/CharacterTransferTest.php | adding another users character clears their main selection | `login::adding_another_users_character_clears_their_main_selection` |
| tests/Feature/Auth/CharacterTransferTest.php | adding another users character keeps an unrelated main selection | `login::adding_another_users_character_keeps_an_unrelated_main_selection` |
| tests/Feature/Auth/CharacterTransferTest.php | taking a users last character deletes the abandoned account | `login::taking_a_users_last_character_deletes_the_abandoned_account` |
| tests/Feature/Auth/CharacterTransferTest.php | re-adding your own character changes nothing | `login::re_adding_your_own_character_changes_nothing` |
| tests/Feature/Auth/LoginScopesTest.php | asks for no scopes at a normal login | `login_scopes::asks_for_no_scopes_at_a_normal_login` |
| tests/Feature/Auth/LoginScopesTest.php | shares a sync grant link asking only for the character contact scopes | `login_scopes::shares_a_sync_grant_link_asking_only_for_the_character_contact_scopes` |
| tests/Feature/Auth/LoginScopesTest.php | shares the admin grant link only with admins | `login_scopes::shares_the_admin_grant_link_only_with_admins` |
| tests/Feature/Auth/LoginScopesTest.php | asks for exactly the scopes a grant link carries | `login_scopes::asks_for_exactly_the_scopes_a_grant_link_carries` |
| tests/Feature/Auth/LoginScopesTest.php | asks for the mail scope through the admin grant link | `login_scopes::asks_for_the_mail_scope_through_the_admin_grant_link` |
| tests/Feature/Auth/LoginScopesTest.php | shows whether the admin character can send expiry mail | `login_scopes::shows_whether_the_admin_character_can_send_expiry_mail` |
| tests/Feature/Console/CheckSyncTokensTest.php | mails synced characters whose write-contacts token expired | `check_tokens::mails_synced_characters_whose_write_contacts_token_expired` |
| tests/Feature/Console/CheckSyncTokensTest.php | does not mail characters that still have a token | `check_tokens::does_not_mail_characters_that_still_have_a_token` |
| tests/Feature/Console/CheckSyncTokensTest.php | does not mail characters that inherit the source | `check_tokens::does_not_mail_characters_that_inherit_the_source` |
| tests/Feature/Console/CheckSyncTokensTest.php | mails each lapsed character at most once | `check_tokens::mails_each_lapsed_character_at_most_once` |
| tests/Feature/Console/CheckSyncTokensTest.php | does nothing when the source character cannot send mail | `check_tokens::does_nothing_when_the_source_character_cannot_send_mail` |
| tests/Feature/Console/CheckSyncTokensTest.php | sends an eve mail from the source character to the lapsed character | `check_tokens::sends_an_eve_mail_from_the_source_character_to_the_lapsed_character` |
| tests/Feature/Console/SyncCharacterAffiliationsCommandTest.php | refreshes the affiliations of user characters and source contacts | `affiliations::refreshes_the_affiliations_of_user_characters_and_source_contacts` |
| tests/Feature/Console/SyncCharacterAffiliationsCommandTest.php | skips the affiliation sync during eve downtime | `affiliations::skips_the_affiliation_sync_during_eve_downtime` |
| tests/Feature/Console/SyncStandingsCommandTest.php | refreshes the source and queues every opted-in character | `standings_sync::refreshes_the_source_and_queues_every_opted_in_character` |
| tests/Feature/Console/SyncStandingsCommandTest.php | does not queue any syncs when the source is unchanged | `standings_sync::does_not_queue_any_syncs_when_the_source_is_unchanged` |
| tests/Feature/Console/SyncStandingsCommandTest.php | queues syncs for unchanged standings when forced | `standings_sync::queues_syncs_for_unchanged_standings_when_forced` |
| tests/Feature/Console/SyncStandingsCommandTest.php | skips the sync during eve downtime without calling esi or alerting | `standings_sync::skips_the_sync_during_eve_downtime_without_calling_esi_or_alerting` |
| tests/Feature/Console/SyncStandingsCommandTest.php | fails when no source is configured | `standings_sync::fails_when_no_source_is_configured` |
| tests/Feature/Console/SyncStandingsCommandTest.php | alerts discord at most once per hour when the source cannot be read | `standings_sync::alerts_discord_at_most_once_per_hour_when_the_source_cannot_be_read` |
| tests/Feature/Console/SyncStandingsCommandTest.php | does not alert discord when no source is configured | `standings_sync::does_not_alert_discord_when_no_source_is_configured` |
| tests/Feature/Console/SyncStandingsCommandTest.php | posts a source-unreadable alert to the discord webhook | `standings_sync::posts_a_source_unreadable_alert_to_the_discord_webhook` |
| tests/Feature/DashboardStandingsTest.php | shows the source standings and the user characters on the dashboard | `dashboard::shows_the_source_standings_and_the_user_characters_on_the_dashboard` |
| tests/Feature/DashboardStandingsTest.php | flags a character standing as redundant when its alliance carries the same standing | `dashboard::flags_a_character_standing_as_redundant_when_its_alliance_carries_the_same_standing` |
| tests/Feature/DashboardStandingsTest.php | flags a corporation standing as redundant when its alliance carries the same standing | `dashboard::flags_a_corporation_standing_as_redundant_when_its_alliance_carries_the_same_standing` |
| tests/Feature/DashboardStandingsTest.php | flags a weaker standing that its parent covers in the same direction (positive parent covers weaker positive) | `dashboard::flags_a_weaker_standing_positive_parent_covers_weaker_positive` |
| tests/Feature/DashboardStandingsTest.php | flags a weaker standing that its parent covers in the same direction (negative parent covers weaker negative) | `dashboard::flags_a_weaker_standing_negative_parent_covers_weaker_negative` |
| tests/Feature/DashboardStandingsTest.php | does not flag a standing its parent does not cover (stronger positive than parent) | `dashboard::does_not_flag_a_standing_stronger_positive_than_parent` |
| tests/Feature/DashboardStandingsTest.php | does not flag a standing its parent does not cover (stronger negative than parent) | `dashboard::does_not_flag_a_standing_stronger_negative_than_parent` |
| tests/Feature/DashboardStandingsTest.php | does not flag a standing its parent does not cover (opposite direction of parent) | `dashboard::does_not_flag_a_standing_opposite_direction_of_parent` |
| tests/Feature/DashboardStandingsTest.php | does not flag a standing its parent does not cover (neutral under a blue parent) | `dashboard::does_not_flag_a_standing_neutral_under_a_blue_parent` |
| tests/Feature/DashboardStandingsTest.php | prefers the corporation over the alliance when both make a character redundant | `dashboard::prefers_the_corporation_over_the_alliance_when_both_make_a_character_redundant` |
| tests/Feature/DashboardStandingsTest.php | offers a request option for a character with no standing | `dashboard::offers_a_request_option_for_a_character_with_no_standing` |
| tests/Feature/DashboardStandingsTest.php | exposes the corporation entity as a deduplicated request option | `dashboard::exposes_the_corporation_entity_as_a_deduplicated_request_option` |
| tests/Feature/DashboardTest.php | guests are redirected to the login page | `dashboard::guests_are_redirected_to_the_login_page` |
| tests/Feature/DashboardTest.php | authenticated users can visit the dashboard | `dashboard::authenticated_users_can_visit_the_dashboard` |
| tests/Feature/ExampleTest.php | returns a successful response | `pages::returns_a_successful_response` |
| tests/Feature/MainCharacterTest.php | users can set their main character | `characters::users_can_set_their_main_character` |
| tests/Feature/MainCharacterTest.php | users cannot set a character they do not own as main | `characters::users_cannot_set_a_character_they_do_not_own_as_main` |
| tests/Feature/MainCharacterTest.php | removing the main character clears the main selection | `characters::removing_the_main_character_clears_the_main_selection` |
| tests/Feature/MainCharacterTest.php | removing an alt keeps the main selection | `characters::removing_an_alt_keeps_the_main_selection` |
| tests/Feature/MainCharacterTest.php | the dashboard marks the main character | `characters::the_dashboard_marks_the_main_character` |
| tests/Feature/ScheduleTest.php | schedules the standings sync every five minutes | `scheduler::schedules_the_standings_sync_every_five_minutes` |
| tests/Feature/ScheduleTest.php | schedules the affiliation sync hourly | `scheduler::schedules_the_affiliation_sync_hourly` |
| tests/Feature/ScheduleTest.php | schedules the token check daily | `scheduler::schedules_the_token_check_daily` |
| tests/Feature/SetupWizardTest.php | the setup wizard is shown to a user without a main character | `setup::the_setup_wizard_is_shown_to_a_user_without_a_main_character` |
| tests/Feature/SetupWizardTest.php | the setup wizard is not shown once a main character is selected | `setup::the_setup_wizard_is_not_shown_once_a_main_character_is_selected` |
| tests/Feature/SetupWizardTest.php | the setup wizard is not shown after it was skipped | `setup::the_setup_wizard_is_not_shown_after_it_was_skipped` |
| tests/Feature/SetupWizardTest.php | selecting a main character completes the setup | `setup::selecting_a_main_character_completes_the_setup` |
| tests/Feature/SetupWizardTest.php | completing the setup does not overwrite an earlier completion time | `setup::completing_the_setup_does_not_overwrite_an_earlier_completion_time` |
| tests/Feature/StandingRequests/EffectiveStandingResolverTest.php | resolves a standing set directly on the entity | `standing_requests::resolves_a_standing_set_directly_on_the_entity` |
| tests/Feature/StandingRequests/EffectiveStandingResolverTest.php | inherits a standing from the requesting characters corporation | `standing_requests::inherits_a_standing_from_the_requesting_characters_corporation` |
| tests/Feature/StandingRequests/EffectiveStandingResolverTest.php | prefers a direct standing over an inherited one | `standing_requests::prefers_a_direct_standing_over_an_inherited_one` |
| tests/Feature/StandingRequests/EffectiveStandingResolverTest.php | returns null when neither the entity nor a parent has a standing | `standing_requests::returns_null_when_neither_the_entity_nor_a_parent_has_a_standing` |
| tests/Feature/StandingRequests/StandingRequestTest.php | lets a user request standing for their character | `standing_requests::lets_a_user_request_standing_for_their_character` |
| tests/Feature/StandingRequests/StandingRequestTest.php | lets a user request standing for their corporation | `standing_requests::lets_a_user_request_standing_for_their_corporation` |
| tests/Feature/StandingRequests/StandingRequestTest.php | does not allow a request when the subject already has standing | `standing_requests::does_not_allow_a_request_when_the_subject_already_has_standing` |
| tests/Feature/StandingRequests/StandingRequestTest.php | forbids requesting standing for another users character | `standing_requests::forbids_requesting_standing_for_another_users_character` |
| tests/Feature/StandingRequests/StandingRequestTest.php | does not duplicate a pending request for the same subject | `standing_requests::does_not_duplicate_a_pending_request_for_the_same_subject` |
| tests/Feature/StandingRequests/StandingRequestTest.php | posts a standing request to the configured discord webhook and pings the role | `standing_requests::posts_a_standing_request_to_the_configured_discord_webhook_and_pings_the_role` |
| tests/Feature/StandingRequests/StandingRequestTest.php | skips the discord webhook when none is configured | `standing_requests::skips_the_discord_webhook_when_none_is_configured` |
| tests/Feature/Standings/CharacterStandingsSyncTest.php | adds new source contacts to a syncing character | `character_sync::adds_new_source_contacts_to_a_syncing_character` |
| tests/Feature/Standings/CharacterStandingsSyncTest.php | updates contacts whose standing changed | `character_sync::updates_contacts_whose_standing_changed` |
| tests/Feature/Standings/CharacterStandingsSyncTest.php | does not write contacts that already match the source | `character_sync::does_not_write_contacts_that_already_match_the_source` |
| tests/Feature/Standings/CharacterStandingsSyncTest.php | removes a previously synced contact once the source drops it | `character_sync::removes_a_previously_synced_contact_once_the_source_drops_it` |
| tests/Feature/Standings/CharacterStandingsSyncTest.php | never deletes personal contacts that were never synced | `character_sync::never_deletes_personal_contacts_that_were_never_synced` |
| tests/Feature/Standings/CharacterStandingsSyncTest.php | skips characters that have not opted in | `character_sync::skips_characters_that_have_not_opted_in` |
| tests/Feature/Standings/CharacterStandingsSyncTest.php | skips characters without the write scope | `character_sync::skips_characters_without_the_write_scope` |
| tests/Feature/Standings/CharacterStandingsSyncTest.php | skips characters that already inherit the source corporation standings | `character_sync::skips_characters_that_already_inherit_the_source_corporation_standings` |
| tests/Feature/Standings/CharacterSyncToggleTest.php | lets a user re-sync only their own opted-in characters | `characters::lets_a_user_re_sync_only_their_own_opted_in_characters` |
| tests/Feature/Standings/CharacterSyncToggleTest.php | lets a user enable syncing for their own character | `characters::lets_a_user_enable_syncing_for_their_own_character` |
| tests/Feature/Standings/CharacterSyncToggleTest.php | lets a user disable syncing for their own character | `characters::lets_a_user_disable_syncing_for_their_own_character` |
| tests/Feature/Standings/CharacterSyncToggleTest.php | forbids toggling a character belonging to another user | `characters::forbids_toggling_a_character_belonging_to_another_user` |
| tests/Feature/Standings/CharacterSyncToggleTest.php | requires authentication to toggle syncing | `characters::requires_authentication_to_toggle_syncing` |
| tests/Feature/Standings/StandingsSourceServiceTest.php | fetches and stores corporation source contacts | `standings_source::fetches_and_stores_corporation_source_contacts` |
| tests/Feature/Standings/StandingsSourceServiceTest.php | stores contact entities locally with their affiliations | `standings_source::stores_contact_entities_locally_with_their_affiliations` |
| tests/Feature/Standings/StandingsSourceServiceTest.php | removes source contacts the source no longer has | `standings_source::removes_source_contacts_the_source_no_longer_has` |
| tests/Feature/Standings/StandingsSourceServiceTest.php | does nothing when no source is configured | `standings_source::does_nothing_when_no_source_is_configured` |
| tests/Feature/Standings/StandingsSourceServiceTest.php | does nothing when no admin can read the source | `standings_source::does_nothing_when_no_admin_can_read_the_source` |
| tests/Feature/Standings/StandingsSourceServiceTest.php | marks a pending request done when a direct blue standing appears | `standings_source::marks_a_pending_request_done_when_a_direct_blue_standing_appears` |
| tests/Feature/Standings/StandingsSourceServiceTest.php | does not close a request when the direct standing is neutral or red | `standings_source::does_not_close_a_request_when_the_direct_standing_is_neutral_or_red` |
| tests/Feature/Standings/StandingsSourceServiceTest.php | does not auto-close a request that is only blue through a parent | `standings_source::does_not_auto_close_a_request_that_is_only_blue_through_a_parent` |
| tests/Feature/Standings/StandingsSourceServiceTest.php | reports no change and skips name resolution when the standings are unchanged | `standings_source::reports_no_change_and_skips_name_resolution_when_the_standings_are_unchanged` |
| tests/Feature/StandingsVisibilityTest.php | hides the standings from users without an eligible character | `visibility::hides_the_standings_from_users_without_an_eligible_character` |
| tests/Feature/StandingsVisibilityTest.php | shows the standings to a user whose character is covered by the source | `visibility::shows_the_standings_to_a_user_whose_character_is_covered_by_the_source` |
| tests/Feature/StandingsVisibilityTest.php | shows the standings to a user with a directly blue character | `visibility::shows_the_standings_to_a_user_with_a_directly_blue_character` |
| tests/Feature/StandingsVisibilityTest.php | shows the standings to a user blue via their alliance | `visibility::shows_the_standings_to_a_user_blue_via_their_alliance` |
| tests/Feature/StandingsVisibilityTest.php | keeps the standings hidden from a red character | `visibility::keeps_the_standings_hidden_from_a_red_character` |
| tests/Feature/StandingsVisibilityTest.php | always shows the standings to admins | `visibility::always_shows_the_standings_to_admins` |
| tests/Feature/StandingsVisibilityTest.php | still offers request options to ineligible users | `visibility::still_offers_request_options_to_ineligible_users` |
| tests/Feature/UserCharacterTest.php | users can switch their active character | `characters::users_can_switch_their_active_character` |
| tests/Feature/UserCharacterTest.php | users cannot switch to a character they do not own | `characters::users_cannot_switch_to_a_character_they_do_not_own` |
| tests/Feature/UserCharacterTest.php | users can remove a character from their account | `characters::users_can_remove_a_character_from_their_account` |
| tests/Feature/UserCharacterTest.php | removing last character deletes the user account | `characters::removing_last_character_deletes_the_user_account` |
| tests/Feature/UserCharacterTest.php | users cannot remove a character they do not own | `characters::users_cannot_remove_a_character_they_do_not_own` |
| tests/Feature/UserCharacterTest.php | active character switches to remaining character after removal | `characters::active_character_switches_to_remaining_character_after_removal` |
| tests/Unit/ExampleTest.php | that true is true | dropped: the starter kit placeholder (`expect(true)->toBeTrue()`) |

Notes on the ports:

- `Queue::fake()` becomes the queued rows in `jobs`; the suites run no
  workers unless they call `jobs::run_pending`.
- `Http::fake()` becomes the mock ESI, EVE SSO and Discord servers on
  ephemeral ports (`tests/integration/common/mocks.rs`).
- The service calls of AffiliationSyncTest and CharacterTransferTest
  (`resolveCharacter`, `addToAccount`) go through the EVE login itself,
  against the mock SSO.
- EffectiveStandingResolverTest's resolver is read through the admin's
  request list, where its result is shown.
- Pest's `assertRedirect()` after a form becomes the form's JSON answer
  (FIXES.md, "Forms answer JSON").

## New tests

Areas the PHP suite never covered:

- `golden_pages`, `golden_mutations`, `golden_background`: every page,
  form, command and job recorded from the Laravel reference
  (tests/fixtures/golden), compared whole.
- `harness`: the route contract (every route of the Laravel app exists).
- `login`: the callback (session, redirects to the intended page, flash
  messages, tokens with their scopes, a new EVE account, failures).
- `esi`, `esi_tokens`: headers, retries, pagination, chunks, the token
  refresh and the token deletion rules.
- `jobs`: one try, failures kept, no job taken twice.
- `scheduler`: wall clock alignment and the downtime window.
- `sessions`: the session's lifetime, the same-origin check, guest forms.
- `pages`: shared props, the flash, error pages, `/settings`, health,
  stale Inertia tabs, trailing slashes.
- `demo`: the demo login and seed.
- `frontend/e2e/routes.spec.ts`, `seo.spec.ts`, `demo.spec.ts`: the route
  contract, heads and the demo in a browser.
- vitest suites next to the frontend code.

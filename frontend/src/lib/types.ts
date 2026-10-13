// The props the api serves, key for key (the Inertia props of the Laravel
// pages).

/** A corporation or alliance as the characters carry it. */
export type EntitySummary = { id: number; name: string | null; ticker: string | null };

/** `CharacterResource`. */
export type Character = {
  id: number;
  name: string | null;
  corporation_id: number | null;
  alliance_id: number | null;
  corporation?: EntitySummary | null;
  alliance?: EntitySummary | null;
};

/** The shared `auth` prop: only `user` for a guest. */
export type Auth = {
  user: { id: number; name: string } | null;
  is_admin?: boolean;
  source_unreadable?: boolean;
  reauth_characters?: { id: number; name: string | null }[];
  sync_scopes_url?: string;
  admin_scopes_url?: string | null;
  active_character?: Character;
  characters?: Character[];
};

/** The flash message of the last form or login. */
export type Flash = { success: string | null; error: string | null; info: string | null };

/** The props every page gets. */
export type SharedProps = {
  name: string;
  auth: Auth;
  sidebarOpen: boolean;
  flash: Flash;
};

/** A link of a Laravel paginator. */
export type PaginationLink = {
  url: string | null;
  label: string;
  page?: number | null;
  active: boolean;
};

/** A Laravel `LengthAwarePaginator`. */
export type Paginated<T> = {
  current_page: number;
  data: T[];
  first_page_url: string;
  from: number | null;
  last_page: number;
  last_page_url: string;
  links: PaginationLink[];
  next_page_url: string | null;
  path: string;
  per_page: number;
  prev_page_url: string | null;
  to: number | null;
  total: number;
};

/** A standings source as the pages show it. */
export type SourceSummary = {
  type: string;
  entity_id: number;
  entity_name: string | null;
  last_synced_at: string | null;
};

/** A source contact on the dashboard. */
export type Standing = {
  contact_id: number;
  contact_type: string;
  name: string | null;
  standing: number;
  redundant_via: { contact_type: string; contact_id: number; name: string | null } | null;
};

/** A character of the dashboard, with its sync state. */
export type SyncCharacter = {
  id: number;
  name: string;
  is_main: boolean;
  should_sync: boolean;
  has_write_scope: boolean;
  inherits_source: boolean;
  synced_contacts_count: number;
  corporation: EntitySummary | null;
  alliance: EntitySummary | null;
};

/** An entity the user may request a standing for. */
export type RequestOption = {
  type: string;
  id: number;
  name: string | null;
  via_character_id: number;
  status: string | null;
};

/** The standing a request's subject already has. */
export type EffectiveStanding = {
  standing: number;
  source: 'direct' | 'corporation' | 'alliance';
  via_type: string;
  via_id: number;
  via_name: string | null;
};

/** A standing request in the admin lists. */
export type StandingRequestItem = {
  id: number;
  status: string;
  created_at: string;
  subject: { type: string; id: number; name: string | null };
  requested_by: string;
  effective_standing: EffectiveStanding | null;
};

/** The standing that applies to a pilot or a group. */
export type PilotStanding = { value: number; source: string };

/** A character row of the admin's pilot list. */
export type PilotCharacter = {
  id: number;
  name: string | null;
  is_main: boolean;
  account: { id: number | null; name: string | null };
  standing: PilotStanding | null;
  corporation: EntitySummary | null;
  alliance: EntitySummary | null;
};

/** A corporation or alliance with its registered characters. */
export type AffiliationGroup = {
  id: number;
  name: string | null;
  ticker: string | null;
  standing: PilotStanding | null;
  characters: PilotCharacter[];
};

/** A source type the admin may pick. */
export type SourceTypeOption = {
  value: string;
  label: string;
  entity_id: number | null;
  entity_name: string | null;
  has_scope: boolean;
  available: boolean;
};

export type HomeProps = SharedProps & { demoLoginEnabled?: boolean };

export type DashboardProps = SharedProps & {
  showSetupWizard: boolean;
  source: SourceSummary | null;
  canViewStandings: boolean;
  standings: Standing[] | null;
  characters: SyncCharacter[];
  requestableOptions: RequestOption[];
};

export type OverviewProps = SharedProps & {
  source: SourceSummary | null;
  stats: {
    pending_requests: number;
    pilots: number;
    syncing_characters: number;
    source_contacts: number;
  };
  recentRequests: StandingRequestItem[];
};

export type StandingRequestsProps = SharedProps & {
  standingRequests: Paginated<StandingRequestItem>;
  counts: { pending: number; done: number; rejected: number };
  filters: { status: string };
};

export type PilotsProps = SharedProps & {
  characters: PilotCharacter[] | null;
  groups: Paginated<AffiliationGroup> | null;
  filters: { search: string; view: string };
};

export type AdminSettingsProps = SharedProps & {
  source: SourceSummary | null;
  sourceTypes: SourceTypeOption[];
  adminCharacter: { id: number; name: string | null; has_mail_scope: boolean } | null;
  grantMailScopeUrl: string;
  contactsCount: number;
  discordSettings: { webhook_url: string | null; role_id: string | null };
};

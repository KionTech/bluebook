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

export type BreadcrumbItem = { title: string; href: string };

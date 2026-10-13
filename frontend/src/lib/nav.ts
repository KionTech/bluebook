// The sections of the top bar and the admin pages, and which one the
// current path belongs to.

export type NavItem = { title: string; href: string };

/** The top bar's sections: Admin only for admins. */
export function primarySections(isAdmin: boolean): NavItem[] {
  return [
    { title: 'Standings', href: '/dashboard' },
    ...(isAdmin ? [{ title: 'Admin', href: '/admin' }] : []),
  ];
}

export const ADMIN_SECTIONS: NavItem[] = [
  { title: 'Overview', href: '/admin' },
  { title: 'Requests', href: '/admin/standing-requests' },
  { title: 'Pilots', href: '/admin/pilots' },
  { title: 'Settings', href: '/admin/settings' },
];

function trimSlash(path: string): string {
  return path.length > 1 ? path.replace(/\/+$/, '') : path;
}

/** Whether a top bar section covers the path, its own pages included. */
export function isSectionActive(href: string, path: string): boolean {
  const clean = trimSlash(path);
  return clean === href || clean.startsWith(`${href}/`);
}

/** Whether an admin tab is the current page: Overview only on /admin. */
export function isTabActive(href: string, path: string): boolean {
  return trimSlash(path) === href;
}

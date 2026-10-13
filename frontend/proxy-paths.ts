// The shared-origin routing contract, mirrored by deploy/Caddyfile: keep
// both in sync when a path changes.

// Paths Axum owns outright: every method proxies to the API. `/api` the
// page props and the health check, `/up` the health page, `/eve` the EVE
// login and its callback, `/demo-login` the local demo login, `/storage`
// the signed file links.
export const axumPrefixes = ['/api', '/up', '/eve', '/demo-login', '/storage'];

// `/settings` itself (a redirect to the appearance page, for every method)
// is Axum's; the page below it, `/settings/appearance`, is
// SvelteKit's.
const axumExact = new Set(['/settings']);

// The forms post to the URLs they always used (PUT /standings/3, POST
// /logout, ...): every request that is not a GET or HEAD goes to Axum as
// well.
export function isAxumRequest(method: string, path: string): boolean {
  if (method !== 'GET' && method !== 'HEAD') {
    return true;
  }
  return (
    axumExact.has(path) ||
    axumPrefixes.some((prefix) => path === prefix || path.startsWith(`${prefix}/`))
  );
}

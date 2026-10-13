// The links of a paginated list: absolute URLs on the site, shown with
// their arrows decoded.

export function linkLabel(label: string): string {
  return label.replace('&laquo;', '«').replace('&raquo;', '»');
}

/** A paginator URL as a path on the site, so SvelteKit navigates to it. */
export function sitePath(url: string): string {
  const parsed = new URL(url, 'http://site.invalid');
  return `${parsed.pathname}${parsed.search}`;
}

// The document title the way Inertia's title callback built it:
// "<title> - <app name>", or the app name alone.
export function pageTitle(title: string | undefined, appName: string): string {
  return title ? `${title} - ${appName}` : appName;
}

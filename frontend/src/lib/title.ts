// The document title: "<title> - <app name>", or the app name alone.
export function pageTitle(title: string | undefined, appName: string): string {
  return title ? `${title} - ${appName}` : appName;
}

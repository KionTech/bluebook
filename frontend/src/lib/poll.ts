// Inertia's usePoll: reload the page's data every interval while the tab
// is visible. Returns the stop function.
export function startPolling(
  reload: () => unknown,
  intervalMs: number,
  isVisible: () => boolean = () => document.visibilityState === "visible",
): () => void {
  const timer = setInterval(() => {
    if (isVisible()) {
      reload();
    }
  }, intervalMs);
  return () => clearInterval(timer);
}

/** How often the dashboard and the admin pages reload. */
export const POLL_INTERVAL_MS = 30_000;

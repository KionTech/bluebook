import { afterEach, describe, expect, it, vi } from "vitest";

import { POLL_INTERVAL_MS, startPolling } from "./poll.ts";

describe("startPolling", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("reloads every interval while the tab is visible, until stopped", () => {
    vi.useFakeTimers();
    let visible = true;
    const reload = vi.fn();
    const stop = startPolling(reload, POLL_INTERVAL_MS, () => visible);

    vi.advanceTimersByTime(29_999);
    expect(reload).toHaveBeenCalledTimes(0);
    vi.advanceTimersByTime(1);
    expect(reload).toHaveBeenCalledTimes(1);

    visible = false;
    vi.advanceTimersByTime(30_000);
    expect(reload).toHaveBeenCalledTimes(1);

    visible = true;
    stop();
    vi.advanceTimersByTime(60_000);
    expect(reload).toHaveBeenCalledTimes(1);
  });
});

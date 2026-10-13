import { describe, expect, it } from "vitest";

import { statusFilterOf, statusVariant } from "./requests.ts";

describe("requests", () => {
  it("falls back to the pending filter", () => {
    expect(statusFilterOf("done")).toBe("done");
    expect(statusFilterOf("rejected")).toBe("rejected");
    expect(statusFilterOf("maybe")).toBe("pending");
  });

  it("badges a resolved request by its status", () => {
    expect(statusVariant("done")).toBe("secondary");
    expect(statusVariant("rejected")).toBe("destructive");
    expect(statusVariant("pending")).toBe("outline");
  });
});

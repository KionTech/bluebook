import { describe, expect, it } from "vitest";

import { STORAGE_KEY, storedStep } from "./wizard.ts";

describe("storedStep", () => {
  const storage = (value: string | null) => ({
    getItem: (key: string) => (key === STORAGE_KEY ? value : null),
  });

  it("resumes a stored step and starts over on anything else", () => {
    expect(storedStep(storage("2"))).toBe(2);
    expect(storedStep(storage(null))).toBe(0);
    expect(storedStep(storage("4"))).toBe(0);
    expect(storedStep(storage("-1"))).toBe(0);
    expect(storedStep(storage("x"))).toBe(0);
  });
});

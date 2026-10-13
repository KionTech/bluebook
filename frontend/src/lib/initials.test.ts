import { describe, expect, it } from "vitest";

import { getInitials } from "./initials.ts";

describe("getInitials", () => {
  it("takes the first and last initial", () => {
    expect(getInitials("Nicolas Pilot")).toBe("NP");
    expect(getInitials("admin alt")).toBe("AA");
    expect(getInitials("Tina")).toBe("T");
    expect(getInitials("Kion Trading Inc.")).toBe("KI");
    expect(getInitials(null)).toBe("");
    expect(getInitials("")).toBe("");
  });
});

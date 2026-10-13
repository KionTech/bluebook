import { describe, expect, it } from "vitest";

import { pagesPath, sitePath } from "./api.ts";

describe("pagesPath", () => {
  it("maps a page URL to its props endpoint", () => {
    expect(pagesPath(new URL("https://bluebook.nicolaskion.dev/"))).toBe(
      "/api/pages/home",
    );
    expect(
      pagesPath(new URL("https://bluebook.nicolaskion.dev/dashboard")),
    ).toBe("/api/pages/dashboard");
    expect(
      pagesPath(
        new URL(
          "https://bluebook.nicolaskion.dev/admin/pilots?view=alliances&page=2",
        ),
      ),
    ).toBe("/api/pages/admin/pilots?view=alliances&page=2");
  });
});

describe("sitePath", () => {
  it("keeps the redirect target", () => {
    expect(sitePath("/eve")).toBe("/eve");
    expect(sitePath(null)).toBe("/");
  });
});

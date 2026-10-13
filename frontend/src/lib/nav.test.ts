import { describe, expect, it } from "vitest";

import {
  ADMIN_SECTIONS,
  isSectionActive,
  isTabActive,
  primarySections,
} from "./nav.ts";

describe("primarySections", () => {
  it("shows Standings to everyone and Admin only to admins", () => {
    expect(primarySections(false).map((item) => item.title)).toEqual([
      "Standings",
    ]);
    expect(primarySections(true).map((item) => item.href)).toEqual([
      "/dashboard",
      "/admin",
    ]);
  });
});

describe("isSectionActive", () => {
  it("marks Admin on every admin page", () => {
    expect(isSectionActive("/admin", "/admin")).toBe(true);
    expect(isSectionActive("/admin", "/admin/pilots")).toBe(true);
    expect(isSectionActive("/admin", "/administrator")).toBe(false);
  });

  it("ignores a trailing slash", () => {
    expect(isSectionActive("/dashboard", "/dashboard/")).toBe(true);
    expect(isSectionActive("/dashboard", "/admin")).toBe(false);
  });
});

describe("isTabActive", () => {
  it("marks Overview only on the overview itself", () => {
    expect(isTabActive("/admin", "/admin")).toBe(true);
    expect(isTabActive("/admin", "/admin/pilots")).toBe(false);
    expect(isTabActive("/admin/pilots", "/admin/pilots/")).toBe(true);
  });

  it("has a tab for every admin page", () => {
    expect(ADMIN_SECTIONS.map((item) => item.href)).toEqual([
      "/admin",
      "/admin/standing-requests",
      "/admin/pilots",
      "/admin/settings",
    ]);
  });
});

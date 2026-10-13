import { describe, expect, it } from "vitest";

import type { SyncCharacter } from "#lib/types.ts";

import { otherCharacters, syncState, syncSummary } from "./characters.ts";

function character(overrides: Partial<SyncCharacter> = {}): SyncCharacter {
  return {
    id: 1,
    name: "Pilot",
    is_main: false,
    should_sync: false,
    has_write_scope: true,
    inherits_source: false,
    synced_contacts_count: 0,
    corporation: null,
    alliance: null,
    ...overrides,
  };
}

const none = new Set<number>();

describe("syncState", () => {
  it("marks a character that inherits the source, whatever its token", () => {
    expect(
      syncState(
        character({ inherits_source: true, has_write_scope: false }),
        none,
      ),
    ).toBe("inherited");
  });

  it("asks a syncing character without the write scope to log in again", () => {
    expect(
      syncState(character({ should_sync: true, has_write_scope: false }), none),
    ).toBe("reauth");
  });

  it("asks a character the api lists for re-authentication to log in again", () => {
    expect(syncState(character({ id: 7 }), new Set([7]))).toBe("reauth");
  });

  it("offers a first grant to a character that never had the scope", () => {
    expect(syncState(character({ has_write_scope: false }), none)).toBe(
      "grant",
    );
  });

  it("shows the switch otherwise", () => {
    expect(syncState(character(), none)).toBe("toggle");
  });
});

describe("syncSummary", () => {
  it("counts the synced contacts of a syncing character", () => {
    const syncing = character({ should_sync: true, synced_contacts_count: 12 });
    expect(syncSummary(syncing, "toggle")).toBe("12 contacts synced");
  });

  it("describes the other states", () => {
    expect(syncSummary(character(), "toggle")).toBe("Sync off");
    expect(syncSummary(character(), "inherited")).toBe("Inherits the source");
    expect(syncSummary(character(), "reauth")).toBe("Not syncing");
    expect(syncSummary(character(), "grant")).toBe("No contact permissions");
  });
});

describe("otherCharacters", () => {
  it("leaves out the active character", () => {
    expect(otherCharacters([{ id: 1 }, { id: 2 }], 1)).toEqual([{ id: 2 }]);
    expect(otherCharacters(undefined, 1)).toEqual([]);
  });
});

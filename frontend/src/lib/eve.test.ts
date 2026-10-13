import { describe, expect, it } from "vitest";

import {
  characterPortrait,
  effectiveStandingChipClass,
  effectiveStandingTextClass,
  eveImage,
  standingChipClass,
  standingLabel,
  standingTextClass,
} from "./eve.ts";

describe("eveImage", () => {
  it("links portraits and logos, and nothing for factions", () => {
    expect(eveImage("character", 1)).toBe(
      "https://images.evetech.net/characters/1/portrait?size=64",
    );
    expect(eveImage("corporation", 2, 128)).toBe(
      "https://images.evetech.net/corporations/2/logo?size=128",
    );
    expect(eveImage("alliance", 3)).toBe(
      "https://images.evetech.net/alliances/3/logo?size=64",
    );
    expect(eveImage("faction", 500001)).toBeNull();
    expect(characterPortrait(4)).toBe(
      "https://images.evetech.net/characters/4/portrait?size=64",
    );
  });
});

describe("standings", () => {
  it("labels a standing with its sign", () => {
    expect(standingLabel(10)).toBe("+10");
    expect(standingLabel(7.5)).toBe("+7.5");
    expect(standingLabel(0)).toBe("0");
    expect(standingLabel(-5)).toBe("-5");
  });

  it("colours a standing from blue to red", () => {
    expect(standingTextClass(10)).toBe("text-standing-excellent");
    expect(standingTextClass(1)).toBe("text-standing-good");
    expect(standingTextClass(0)).toBe("text-standing-neutral");
    expect(standingTextClass(-1)).toBe("text-standing-bad");
    expect(standingTextClass(-5)).toBe("text-standing-terrible");
    expect(standingChipClass(5)).toBe(
      "bg-standing-excellent/12 text-standing-excellent",
    );
    expect(standingChipClass(-10)).toBe(
      "bg-standing-terrible/12 text-standing-terrible",
    );
  });

  it("shows a source covered entity green", () => {
    expect(effectiveStandingTextClass({ value: 10, source: "source" })).toBe(
      "text-standing-own",
    );
    expect(
      effectiveStandingTextClass({ value: -5, source: "corporation" }),
    ).toBe("text-standing-terrible");
    expect(effectiveStandingChipClass({ value: 10, source: "source" })).toBe(
      "bg-standing-own/12 text-standing-own",
    );
  });
});

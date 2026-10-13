// The admin's flat character list: searched and sorted in the browser.
import type { PilotCharacter } from "#lib/types.ts";

export type SortKey =
  "name" | "account" | "corporation" | "alliance" | "standing";

export const VIEWS = [
  { value: "characters", label: "Characters" },
  { value: "corporations", label: "Corporations" },
  { value: "alliances", label: "Alliances" },
] as const;

export type View = (typeof VIEWS)[number]["value"];

export function viewOf(value: string): View {
  return VIEWS.some((view) => view.value === value)
    ? (value as View)
    : "characters";
}

function sortText(character: PilotCharacter, key: SortKey): string {
  if (key === "account") {
    return character.account.name ?? "";
  }
  if (key === "corporation" || key === "alliance") {
    return character[key]?.name ?? "";
  }
  return character.name ?? "";
}

/** The characters matching the search (name, account, corporation or
 * alliance), sorted by a column. Without a standing a character sorts
 * last either way. */
export function visibleCharacters(
  characters: PilotCharacter[],
  search: string,
  key: SortKey,
  ascending: boolean,
): PilotCharacter[] {
  const term = search.trim().toLowerCase();
  const rows = characters.filter(
    (character) =>
      term === "" ||
      [
        character.name,
        character.account.name,
        character.corporation?.name,
        character.alliance?.name,
      ].some((value) => value?.toLowerCase().includes(term)),
  );
  const direction = ascending ? 1 : -1;
  return rows.toSorted((a, b) => {
    if (key === "standing") {
      if (a.standing === null || b.standing === null) {
        return Number(a.standing === null) - Number(b.standing === null);
      }
      return direction * (a.standing.value - b.standing.value);
    }
    return direction * sortText(a, key).localeCompare(sortText(b, key));
  });
}

/** The query of the pilots page for a view and search. */
export function pilotsUrl(view: View, search: string): string {
  const params = new URLSearchParams();
  if (view !== "characters") {
    params.set("view", view);
  }
  if (search) {
    params.set("search", search);
  }
  const query = params.toString();
  return query ? `/admin/pilots?${query}` : "/admin/pilots";
}

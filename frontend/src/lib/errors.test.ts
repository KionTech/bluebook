import { describe, expect, it } from "vitest";

import { errorMessage, errorTitle } from "./errors.ts";

describe("errorTitle", () => {
  it("names the status like the api", () => {
    expect(errorTitle(403)).toBe("Forbidden");
    expect(errorTitle(404)).toBe("Not Found");
    expect(errorTitle(419)).toBe("Page Expired");
    expect(errorTitle(500)).toBe("Server Error");
    expect(errorTitle(502)).toBe("Server Error");
  });
});

describe("errorMessage", () => {
  it("shows the reason of a 403 and the title of anything else", () => {
    expect(errorMessage(403, "This action is unauthorized.")).toBe(
      "This action is unauthorized.",
    );
    expect(errorMessage(403, null)).toBe("Forbidden");
    expect(errorMessage(404, "The route x could not be found.")).toBe(
      "Not Found",
    );
  });
});

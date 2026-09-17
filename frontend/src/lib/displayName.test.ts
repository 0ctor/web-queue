import { describe, expect, it } from "vitest";
import { formatDisplayName, normalizeDisplayCode } from "./displayName";

describe("formatDisplayName", () => {
  it("usa primeiro nome e inicial no default LGPD", () => {
    expect(formatDisplayName("Maria Silva", "first_last_initial")).toBe(
      "Maria S.",
    );
  });
});

describe("normalizeDisplayCode", () => {
  it("normaliza código de 6 caracteres", () => {
    expect(normalizeDisplayCode("ab-12c9")).toBe("AB12C9");
  });
});

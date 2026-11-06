import { describe, expect, it } from "vitest";
import { suggestSlug } from "../utils/slug";

describe("suggestSlug", () => {
  it("joins lowercase words with hyphens", () => {
    expect(suggestSlug("Hello, World: A First Post!")).toBe("hello-world-a-first-post");
  });

  it("removes accents and other symbols", () => {
    expect(suggestSlug("Crème brûlée — 100% déjà vu")).toBe("creme-brulee-100-deja-vu");
  });

  it("returns an empty slug for a title without letters or digits", () => {
    expect(suggestSlug("¿¡ — !?")).toBe("");
  });

  it("cuts a long title at a word boundary within 100 characters", () => {
    const slug = suggestSlug("word ".repeat(40));
    expect(slug.length).toBeLessThanOrEqual(100);
    expect(slug.endsWith("-")).toBe(false);
    expect(slug.startsWith("word-word")).toBe(true);
  });

  it("changes a reserved name", () => {
    expect(suggestSlug("API")).toBe("api-page");
    expect(suggestSlug("Tags")).toBe("tags-page");
  });
});

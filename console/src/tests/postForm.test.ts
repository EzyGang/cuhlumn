import { describe, expect, it } from "vitest";
import type { Post } from "../types/api/Post";
import { ctaItems, emptyPostForm, formToPostInput, postToForm } from "../utils/postForm";

const backups = (show_cta: boolean | null): Post => ({
  slug: "sqlite-backups",
  title: "How we back up SQLite",
  description: "",
  body: "We back up one file.",
  tags: ["Databases"],
  kind: "post",
  status: "published",
  cover: null,
  og_image: null,
  show_cta,
  published_at: 1_700_000_000,
  updated_at: 1_700_000_000,
  url: "https://harbor.example.com/sqlite-backups/",
});

describe("post call to action", () => {
  it("maps null, true, and false to the three choices and back", () => {
    for (const [value, choice] of [
      [null, "site"],
      [true, "show"],
      [false, "hide"],
    ] as const) {
      const form = postToForm(backups(value));
      expect(form.show_cta).toBe(choice);
      expect(formToPostInput(form, "published").show_cta).toBe(value);
    }
  });

  it("starts a new post with the site default", () => {
    expect(formToPostInput(emptyPostForm(), "draft").show_cta).toBeNull();
  });

  it("names the site default in the first choice", () => {
    expect(ctaItems(true)[0]?.label).toBe("Site default (shown)");
    expect(ctaItems(false)[0]?.label).toBe("Site default (hidden)");
    expect(ctaItems(null)[0]?.label).toBe("Site default");
    expect(ctaItems(true).map((item) => item.value)).toEqual(["site", "show", "hide"]);
  });
});

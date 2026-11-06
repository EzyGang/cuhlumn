import { describe, expect, it } from "vitest";
import { ALIAS_RULES, TAG_RULES, addChips } from "../utils/chips";

describe("addChips", () => {
  it("splits a draft at commas and trims each tag", () => {
    expect(addChips([], " rust ,  web,, ", TAG_RULES)).toEqual({
      values: ["rust", "web"],
      error: null,
    });
  });

  it("keeps the first spelling of a repeated tag", () => {
    expect(addChips(["Rust"], "rust, RUST, Go", TAG_RULES).values).toEqual(["Rust", "Go"]);
  });

  it("stops at 20 tags and reports the limit", () => {
    const current = Array.from({ length: 19 }, (_, index) => `tag${index}`);
    const result = addChips(current, "last, extra", TAG_RULES);
    expect(result.values).toHaveLength(20);
    expect(result.values.at(-1)).toBe("last");
    expect(result.error).toBe("The limit is 20 tags.");
  });

  it("accepts a repeated tag when the list is full", () => {
    const current = Array.from({ length: 20 }, (_, index) => `tag${index}`);
    expect(addChips(current, "TAG3", TAG_RULES)).toEqual({ values: current, error: null });
  });

  it("rejects a tag longer than 50 characters", () => {
    const result = addChips(["a"], "x".repeat(51), TAG_RULES);
    expect(result).toEqual({ values: ["a"], error: "A tag has at most 50 characters." });
  });

  it("writes aliases in lowercase", () => {
    expect(addChips([], "WWW.Example.com", ALIAS_RULES).values).toEqual(["www.example.com"]);
  });
});

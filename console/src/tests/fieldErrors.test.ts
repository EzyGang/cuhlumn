import { describe, expect, it } from "vitest";
import { ApiError } from "../services/http";
import type { ErrorCode } from "../types/api/ErrorCode";
import { mapErrorToFields, normalizeFieldPath } from "../utils/fieldErrors";

const apiError = (
  field: string | null,
  message = "is not valid",
  code: ErrorCode = "validation",
): ApiError => new ApiError(400, { error: { code, message, field } });

const KNOWN = ["title", "aliases", "aliases.1", "brand.accent", "brand.links", "brand.links.0.url"];

describe("mapErrorToFields", () => {
  it("puts an error on the field with the same path", () => {
    expect(mapErrorToFields(apiError("brand.accent"), KNOWN)).toEqual({
      fields: { "brand.accent": "is not valid" },
      banner: null,
    });
  });

  it("maps an indexed path to the matching list item", () => {
    expect(mapErrorToFields(apiError("aliases.1"), KNOWN).fields).toEqual({
      "aliases.1": "is not valid",
    });
  });

  it("falls back to the closest known parent path", () => {
    expect(mapErrorToFields(apiError("brand.links.3.label"), KNOWN).fields).toEqual({
      "brand.links": "is not valid",
    });
  });

  it("accepts bracket indexes from serde paths", () => {
    expect(mapErrorToFields(apiError("brand.links[0].url"), KNOWN).fields).toEqual({
      "brand.links.0.url": "is not valid",
    });
  });

  it("removes the repeated field name from the field message", () => {
    const error = apiError("brand.links[0].url", "brand.links[0].url: the URL is not valid");
    expect(mapErrorToFields(error, KNOWN).fields).toEqual({
      "brand.links.0.url": "the URL is not valid",
    });
  });

  it("removes the field name from a banner message", () => {
    const error = apiError("brand.head", "brand.head: is too long");
    expect(mapErrorToFields(error, KNOWN).banner).toBe("is too long");
  });

  it("keeps a message that only starts with a different field name", () => {
    const error = apiError("title", "titles: is too long");
    expect(error.message).toBe("titles: is too long");
  });

  it("shows an unknown field in the banner", () => {
    expect(mapErrorToFields(apiError("brand.head"), KNOWN)).toEqual({
      fields: {},
      banner: "is not valid",
    });
  });

  it("shows an error without a field in the banner", () => {
    const result = mapErrorToFields(apiError(null, "site exists", "conflict"), KNOWN);
    expect(result).toEqual({ fields: {}, banner: "site exists" });
  });

  it("does not match a field only by a shared text prefix", () => {
    expect(mapErrorToFields(apiError("titles"), KNOWN).banner).toBe("is not valid");
  });

  it("shows the message of a failure that is not an API error", () => {
    expect(mapErrorToFields(new Error("offline"), KNOWN)).toEqual({
      fields: {},
      banner: "offline",
    });
  });
});

describe("normalizeFieldPath", () => {
  it("turns bracket indexes into dotted indexes", () => {
    expect(normalizeFieldPath("[2].url")).toBe("2.url");
    expect(normalizeFieldPath("tags[19]")).toBe("tags.19");
  });
});

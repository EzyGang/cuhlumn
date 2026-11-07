import { describe, expect, it } from "vitest";
import { MIN_TOKEN_LENGTH, generateToken, isTokenLongEnough } from "../utils/token";

describe("generateToken", () => {
  it("returns 64 lowercase hex characters", () => {
    expect(generateToken()).toMatch(/^[0-9a-f]{64}$/);
  });

  it("returns a different token each time", () => {
    const tokens = new Set(Array.from({ length: 20 }, generateToken));
    expect(tokens.size).toBe(20);
  });

  it("returns a token that the server accepts", () => {
    expect(isTokenLongEnough(generateToken())).toBe(true);
  });
});

describe("isTokenLongEnough", () => {
  it("accepts a token at the minimum length and refuses a shorter one", () => {
    expect(isTokenLongEnough("a".repeat(MIN_TOKEN_LENGTH))).toBe(true);
    expect(isTokenLongEnough("a".repeat(MIN_TOKEN_LENGTH - 1))).toBe(false);
    expect(isTokenLongEnough("")).toBe(false);
  });
});

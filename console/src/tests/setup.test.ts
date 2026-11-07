import { describe, expect, it } from "vitest";
import { redirectTarget } from "../utils/setup";

describe("redirectTarget", () => {
  it("sends every page to the setup page while the server has no token", () => {
    expect(redirectTarget("setup", "/")).toBe("/setup");
    expect(redirectTarget("setup", "/login")).toBe("/setup");
    expect(redirectTarget("setup", "/sites/harbor.example.com/posts")).toBe("/setup");
  });

  it("stays on the setup page while the server has no token", () => {
    expect(redirectTarget("setup", "/setup")).toBeNull();
  });

  it("sends an anonymous visitor from the setup page to the login page", () => {
    expect(redirectTarget("anonymous", "/setup")).toBe("/login");
    expect(redirectTarget("anonymous", "/fonts")).toBe("/login");
    expect(redirectTarget("anonymous", "/login")).toBeNull();
  });

  it("sends a signed-in admin away from the login and setup pages", () => {
    expect(redirectTarget("authenticated", "/setup")).toBe("/");
    expect(redirectTarget("authenticated", "/login")).toBe("/");
    expect(redirectTarget("authenticated", "/settings")).toBeNull();
  });

  it("waits during the startup check", () => {
    expect(redirectTarget("checking", "/setup")).toBeNull();
    expect(redirectTarget("checking", "/")).toBeNull();
  });
});

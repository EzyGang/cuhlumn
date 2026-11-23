import { describe, expect, it } from "vitest";
import type { SiteKind } from "../types/api/SiteKind";
import { analyticsPath, siteHomePath, siteKind } from "../utils/sites";

const sites: { host: string; kind: SiteKind }[] = [
  { host: "harbor.example.com", kind: "blog" },
  { host: "docs.example.org", kind: "external" },
];

describe("site kinds", () => {
  it("finds the kind of a site by its host", () => {
    expect(siteKind(sites, "harbor.example.com")).toBe("blog");
    expect(siteKind(sites, "docs.example.org")).toBe("external");
  });

  it("knows no kind before the list loads or for an unknown host", () => {
    expect(siteKind(null, "docs.example.org")).toBeNull();
    expect(siteKind(sites, "other.example.net")).toBeNull();
    expect(siteKind([], "docs.example.org")).toBeNull();
  });
});

describe("site addresses", () => {
  it("opens a blog at its posts", () => {
    expect(siteHomePath({ host: "harbor.example.com", kind: "blog" })).toBe(
      "/sites/harbor.example.com/posts",
    );
  });

  it("opens an external site at its analytics, because it has no posts", () => {
    expect(siteHomePath({ host: "docs.example.org", kind: "external" })).toBe(
      "/sites/docs.example.org/analytics",
    );
  });

  it("encodes the host as one path segment", () => {
    expect(analyticsPath("docs.example.org/x")).toBe("/sites/docs.example.org%2Fx/analytics");
  });
});

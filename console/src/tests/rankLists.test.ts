import { describe, expect, it } from "vitest";
import type { RankListData } from "../types/analytics";
import type { SiteAnalytics } from "../types/api/SiteAnalytics";
import { rankLists } from "../utils/rankLists";

const report = (): SiteAnalytics => ({
  period: "week",
  from: "2026-11-04",
  to: "2026-11-10",
  views: 200,
  visitors: 120,
  days: [],
  pages: [
    { path: "/sqlite-backups/", title: "How we back up SQLite", views: 150 },
    { path: "/guide/install/", title: null, views: 50 },
  ],
  referrers: [
    { host: null, views: 120 },
    { host: "news.example.org", views: 80 },
  ],
  devices: [{ device: "desktop", views: 200 }],
  browsers: [{ browser: "samsung", views: 200 }],
  outbound: [
    { target: "github.com/EzyGang/cuhlumn", clicks: 30 },
    { target: "pypi.org/project/py-cachify/", clicks: 10 },
  ],
  downloads: [],
});

const list = (title: string): RankListData => {
  const found = rankLists(report()).find((item) => item.title === title);
  if (found === undefined) {
    throw new Error(`the report has no list ${title}`);
  }
  return found;
};

describe("rank lists", () => {
  it("shows the title of a page, and the path alone for a page without a title", () => {
    expect(list("Top pages").rows).toEqual([
      {
        key: "/sqlite-backups/",
        label: "How we back up SQLite",
        secondary: "/sqlite-backups/",
        count: "150",
        share: 75,
      },
      { key: "/guide/install/", label: "/guide/install/", secondary: null, count: "50", share: 25 },
    ]);
  });

  it("names direct visits and browser families", () => {
    expect(list("Top referrers").rows.map((row) => row.label)).toEqual([
      "Direct",
      "news.example.org",
    ]);
    expect(list("Browsers").rows.map((row) => row.label)).toEqual(["Samsung Internet"]);
  });

  it("shows click targets as a share of the clicks, not of the views", () => {
    expect(list("Outbound link clicks").rows).toEqual([
      {
        key: "github.com/EzyGang/cuhlumn",
        label: "github.com/EzyGang/cuhlumn",
        secondary: null,
        count: "30",
        share: 75,
      },
      {
        key: "pypi.org/project/py-cachify/",
        label: "pypi.org/project/py-cachify/",
        secondary: null,
        count: "10",
        share: 25,
      },
    ]);
  });

  it("keeps an empty list, so the report shows that there are no downloads", () => {
    expect(list("Download clicks").rows).toEqual([]);
    expect(list("Download clicks").emptyText).toBe("No data in this period.");
  });
});

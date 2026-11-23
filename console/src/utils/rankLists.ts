import type { RankListData, RankRow } from "../types/analytics";
import type { ClickCount } from "../types/api/ClickCount";
import type { SiteAnalytics } from "../types/api/SiteAnalytics";
import { BROWSER_LABELS, DEVICE_LABELS, referrerLabel } from "./analyticsLabels";
import { formatCount } from "./format";

interface RankSource {
  key: string;
  label: string;
  secondary?: string | null;
  count: number;
}

const rankRows = (sources: readonly RankSource[], total: number): RankRow[] =>
  sources.map((source) => ({
    key: source.key,
    label: source.label,
    secondary: source.secondary ?? null,
    count: formatCount(source.count),
    share: total > 0 ? Math.min(100, (source.count / total) * 100) : 0,
  }));

const pageSources = (report: SiteAnalytics): RankSource[] =>
  report.pages.map((page) => ({
    key: page.path,
    label: page.title ?? page.path,
    secondary: page.title === null ? null : page.path,
    count: page.views,
  }));

const referrerSources = (report: SiteAnalytics): RankSource[] =>
  report.referrers.map((item) => ({
    key: item.host ?? "",
    label: referrerLabel(item.host),
    count: item.views,
  }));

const deviceSources = (report: SiteAnalytics): RankSource[] =>
  report.devices.map((item) => ({
    key: item.device,
    label: DEVICE_LABELS[item.device],
    count: item.views,
  }));

const browserSources = (report: SiteAnalytics): RankSource[] =>
  report.browsers.map((item) => ({
    key: item.browser,
    label: BROWSER_LABELS[item.browser],
    count: item.views,
  }));

const clickSources = (clicks: readonly ClickCount[]): RankSource[] =>
  clicks.map((item) => ({ key: item.target, label: item.target, count: item.clicks }));

const sum = (sources: readonly RankSource[]): number =>
  sources.reduce((total, source) => total + source.count, 0);

/**
 * Builds the ranked lists of a report.
 * A view list shows each row as a share of the period views.
 * A click list shows each row as a share of the clicks in the list, because clicks are not views.
 */
export const rankLists = (report: SiteAnalytics): RankListData[] => {
  const outbound = clickSources(report.outbound);
  const downloads = clickSources(report.downloads);
  const titled: [string, RankSource[], number][] = [
    ["Top pages", pageSources(report), report.views],
    ["Top referrers", referrerSources(report), report.views],
    ["Devices", deviceSources(report), report.views],
    ["Browsers", browserSources(report), report.views],
    ["Outbound link clicks", outbound, sum(outbound)],
    ["Download clicks", downloads, sum(downloads)],
  ];
  return titled.map(([title, sources, total]) => ({
    title,
    emptyText: "No data in this period.",
    rows: rankRows(sources, total),
  }));
};

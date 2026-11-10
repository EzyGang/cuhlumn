import { useSignal } from "@preact/signals";
import { useParams } from "wouter-preact";
import { useLoad } from "../../../hooks/useLoad.hook";
import { getAnalytics } from "../../../services/analytics.service";
import type { AnalyticsModel, RankListData, RankRow } from "../../../types/analytics";
import type { Period } from "../../../types/api/Period";
import type { SiteAnalytics } from "../../../types/api/SiteAnalytics";
import {
  BROWSER_LABELS,
  DEVICE_LABELS,
  PERIOD_ITEMS,
  referrerLabel,
} from "../../../utils/analyticsLabels";
import { longDay } from "../../../utils/chart";
import { dailyChart } from "../../../utils/dailyChart";
import { formatCount } from "../../../utils/format";

interface RankSource {
  key: string;
  label: string;
  secondary?: string | null;
  views: number;
}

const rankRows = (sources: readonly RankSource[], total: number): RankRow[] =>
  sources.map((source) => ({
    key: source.key,
    label: source.label,
    secondary: source.secondary ?? null,
    count: formatCount(source.views),
    share: total > 0 ? Math.min(100, (source.views / total) * 100) : 0,
  }));

const pageSources = (report: SiteAnalytics): RankSource[] =>
  report.pages.map((page) => ({
    key: page.path,
    label: page.title ?? page.path,
    secondary: page.title === null ? null : page.path,
    views: page.views,
  }));

const referrerSources = (report: SiteAnalytics): RankSource[] =>
  report.referrers.map((item) => ({
    key: item.host ?? "",
    label: referrerLabel(item.host),
    views: item.views,
  }));

const deviceSources = (report: SiteAnalytics): RankSource[] =>
  report.devices.map((item) => ({
    key: item.device,
    label: DEVICE_LABELS[item.device],
    views: item.views,
  }));

const browserSources = (report: SiteAnalytics): RankSource[] =>
  report.browsers.map((item) => ({
    key: item.browser,
    label: BROWSER_LABELS[item.browser],
    views: item.views,
  }));

const rankLists = (report: SiteAnalytics): RankListData[] => {
  const titled: [string, RankSource[]][] = [
    ["Top pages", pageSources(report)],
    ["Top referrers", referrerSources(report)],
    ["Devices", deviceSources(report)],
    ["Browsers", browserSources(report)],
  ];
  return titled.map(([title, sources]) => ({
    title,
    emptyText: "No data in this period.",
    rows: rankRows(sources, report.views),
  }));
};

/** Loads the analytics of the current site for the selected period. */
export const useAnalytics = (): AnalyticsModel => {
  const host = useParams<{ host: string }>().host;
  const period = useSignal<Period>("month");
  const loaded = useLoad(`${host}:${period.value}`, (): Promise<SiteAnalytics> =>
    getAnalytics(host, period.value),
  );
  const report = loaded.value;
  return {
    data: {
      period: period.value,
      periodItems: PERIOD_ITEMS,
      rangeText: report === null ? "" : `${longDay(report.from)} to ${longDay(report.to)} (UTC)`,
      views: formatCount(report?.views ?? 0),
      visitors: formatCount(report?.visitors ?? 0),
      empty: (report?.views ?? 0) === 0,
      chart: dailyChart(report?.days ?? []),
      lists: report === null ? [] : rankLists(report),
    },
    status: { load: { status: loaded.status, error: loaded.error, retry: loaded.retry } },
    actions: {
      setPeriod: (next: Period): void => {
        period.value = next;
      },
    },
  };
};

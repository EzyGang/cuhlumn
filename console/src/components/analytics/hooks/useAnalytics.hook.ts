import { useSignal } from "@preact/signals";
import { useParams } from "wouter-preact";
import { useLoad } from "../../../hooks/useLoad.hook";
import { getAnalytics } from "../../../services/analytics.service";
import { sitesStore } from "../../../stores/sites.store";
import type { AnalyticsModel } from "../../../types/analytics";
import type { Period } from "../../../types/api/Period";
import type { SiteAnalytics } from "../../../types/api/SiteAnalytics";
import { PERIOD_ITEMS } from "../../../utils/analyticsLabels";
import { longDay } from "../../../utils/chart";
import { dailyChart } from "../../../utils/dailyChart";
import { formatCount } from "../../../utils/format";
import { rankLists } from "../../../utils/rankLists";
import { siteKind } from "../../../utils/sites";

const BLOG_EMPTY = "No views yet. Views appear here when readers open the blog.";
const EXTERNAL_EMPTY =
  "No views yet. Views appear here when readers open a page that loads the analytics script.";

/** Loads the analytics of the current site for the selected period. */
export const useAnalytics = (): AnalyticsModel => {
  const host = useParams<{ host: string }>().host;
  const period = useSignal<Period>("month");
  const loaded = useLoad(`${host}:${period.value}`, (): Promise<SiteAnalytics> =>
    getAnalytics(host, period.value),
  );
  const report = loaded.value;
  const external = siteKind(sitesStore.sites.value, host) === "external";
  return {
    data: {
      period: period.value,
      periodItems: PERIOD_ITEMS,
      rangeText: report === null ? "" : `${longDay(report.from)} to ${longDay(report.to)} (UTC)`,
      views: formatCount(report?.views ?? 0),
      visitors: formatCount(report?.visitors ?? 0),
      empty: (report?.views ?? 0) === 0,
      emptyText: external ? EXTERNAL_EMPTY : BLOG_EMPTY,
      external,
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

import type { Period } from "./api/Period";
import type { LoadState } from "./load";
import type { SelectItem } from "./select";

/** One bar of the daily chart, in view box units. The slot is the hover area of the bar. */
export interface ChartBar {
  key: string;
  slotX: number;
  slotWidth: number;
  x: number;
  y: number;
  width: number;
  height: number;
  title: string;
}

/** One label on an axis, in view box units. */
export interface ChartLabel {
  key: string;
  position: number;
  text: string;
}

/** One row of the visually hidden table that repeats the chart data. */
export interface ChartRow {
  key: string;
  date: string;
  views: string;
  visitors: string;
}

/** The geometry of the daily chart. */
export interface DailyChartData {
  width: number;
  height: number;
  plot: { left: number; right: number; top: number; bottom: number };
  bars: ChartBar[];
  visitorsLine: string;
  ticks: ChartLabel[];
  days: ChartLabel[];
  rows: ChartRow[];
}

/** One row of a ranked list. */
export interface RankRow {
  key: string;
  label: string;
  secondary: string | null;
  count: string;
  /** The share of the period views, from 0 to 100. */
  share: number;
}

/** A ranked list, for example the top pages. */
export interface RankListData {
  title: string;
  emptyText: string;
  rows: RankRow[];
}

export interface AnalyticsModel {
  data: {
    period: Period;
    periodItems: readonly SelectItem<Period>[];
    rangeText: string;
    views: string;
    visitors: string;
    empty: boolean;
    chart: DailyChartData;
    lists: RankListData[];
  };
  status: { load: LoadState };
  actions: { setPeriod: (period: Period) => void };
}

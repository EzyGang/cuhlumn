import type { ChartBar, ChartLabel, ChartRow, DailyChartData } from "../types/analytics";
import type { DayCount } from "../types/api/DayCount";
import { axisTicks, labelIndexes, longDay, niceMax, shortDay } from "./chart";
import { formatCount } from "./format";

const WIDTH = 720;
const HEIGHT = 240;
const PLOT = { left: 44, right: 712, top: 12, bottom: 212 } as const;
const MAX_DAY_LABELS = 7;

/** Rounds a coordinate to one decimal, so the markup stays short. */
const round = (value: number): number => Math.round(value * 10) / 10;

const scaleY = (value: number, max: number): number =>
  round(PLOT.bottom - (value / max) * (PLOT.bottom - PLOT.top));

const slotOf = (count: number): number => (PLOT.right - PLOT.left) / Math.max(1, count);

const barsOf = (days: readonly DayCount[], max: number): ChartBar[] => {
  const slot = slotOf(days.length);
  const width = slot * (days.length > 90 ? 0.9 : 0.7);
  return days.map((day, index): ChartBar => {
    const y = scaleY(day.views, max);
    const slotX = PLOT.left + index * slot;
    return {
      key: day.date,
      slotX: round(slotX),
      slotWidth: round(slot),
      x: round(slotX + (slot - width) / 2),
      y,
      width: round(width),
      height: round(PLOT.bottom - y),
      title: `${longDay(day.date)}: ${formatCount(day.views)} views, ${formatCount(day.visitors)} visitors`,
    };
  });
};

const centerOf = (index: number, count: number): number =>
  round(PLOT.left + (index + 0.5) * slotOf(count));

const visitorsLineOf = (days: readonly DayCount[], max: number): string =>
  days
    .map((day, index) => `${centerOf(index, days.length)},${scaleY(day.visitors, max)}`)
    .join(" ");

const ticksOf = (top: number, max: number): ChartLabel[] =>
  axisTicks(top).map((tick) => ({
    key: String(tick),
    position: scaleY(tick, max),
    text: formatCount(tick),
  }));

const dayLabelsOf = (days: readonly DayCount[]): ChartLabel[] =>
  labelIndexes(days.length, MAX_DAY_LABELS).map((index) => ({
    key: days[index]?.date ?? String(index),
    position: centerOf(index, days.length),
    text: shortDay(days[index]?.date ?? ""),
  }));

const rowsOf = (days: readonly DayCount[]): ChartRow[] =>
  days.map((day) => ({
    key: day.date,
    date: longDay(day.date),
    views: formatCount(day.views),
    visitors: formatCount(day.visitors),
  }));

/** Builds the geometry of the daily chart: view bars, a visitors line, and the axis labels. */
export const dailyChart = (days: readonly DayCount[]): DailyChartData => {
  const top = Math.max(0, ...days.map((day) => Math.max(day.views, day.visitors)));
  const max = niceMax(top);
  return {
    width: WIDTH,
    height: HEIGHT,
    plot: PLOT,
    bars: barsOf(days, max),
    visitorsLine: visitorsLineOf(days, max),
    ticks: ticksOf(top, max),
    days: dayLabelsOf(days),
    rows: rowsOf(days),
  };
};

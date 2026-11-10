import { describe, expect, it } from "vitest";
import type { DayCount } from "../types/api/DayCount";
import { axisTicks, labelIndexes, longDay, niceMax, niceStep, shortDay } from "../utils/chart";
import { dailyChart } from "../utils/dailyChart";

const days = (views: number[]): DayCount[] =>
  views.map((count, index) => ({
    date: `2026-11-${String(index + 1).padStart(2, "0")}`,
    views: count,
    visitors: Math.ceil(count / 2),
  }));

describe("axis scale", () => {
  it("picks a round step of 1, 2 or 5 times a power of ten", () => {
    expect(niceStep(7)).toBe(2);
    expect(niceStep(100)).toBe(50);
    expect(niceStep(1234)).toBe(500);
  });

  it("never uses a step below one, because counts are whole numbers", () => {
    expect(niceStep(1)).toBe(1);
    expect(niceStep(3)).toBe(1);
  });

  it("rounds the maximum up to a multiple of the step", () => {
    expect(niceMax(7)).toBe(8);
    expect(niceMax(100)).toBe(100);
    expect(niceMax(1234)).toBe(1500);
  });

  it("uses a scale from zero to one when there are no views", () => {
    expect(niceMax(0)).toBe(1);
    expect(axisTicks(0)).toEqual([0, 1]);
  });

  it("lists the ticks from zero to the maximum", () => {
    expect(axisTicks(7)).toEqual([0, 2, 4, 6, 8]);
    expect(axisTicks(100)).toEqual([0, 50, 100]);
  });
});

describe("day labels", () => {
  it("labels the first and the last day and spreads the others", () => {
    expect(labelIndexes(30, 7)).toEqual([0, 5, 10, 15, 19, 24, 29]);
    expect(labelIndexes(7, 7)).toEqual([0, 1, 2, 3, 4, 5, 6]);
  });

  it("handles short series", () => {
    expect(labelIndexes(0, 7)).toEqual([]);
    expect(labelIndexes(1, 7)).toEqual([0]);
    expect(labelIndexes(3, 7)).toEqual([0, 1, 2]);
  });

  it("formats a day in UTC", () => {
    expect(shortDay("2026-11-10", "en-GB")).toBe("10 Nov");
    expect(longDay("2026-11-10", "en-GB")).toBe("10 Nov 2026");
    expect(shortDay("2026-01-01", "en-GB")).toBe("1 Jan");
  });
});

describe("daily chart", () => {
  it("gives one bar and one line point for each day", () => {
    const chart = dailyChart(days([4, 0, 8]));
    expect(chart.bars).toHaveLength(3);
    expect(chart.visitorsLine.split(" ")).toHaveLength(3);
    expect(chart.rows.map((row) => row.views)).toEqual(["4", "0", "8"]);
  });

  it("scales the highest bar to the top of the axis", () => {
    const chart = dailyChart(days([4, 0, 8]));
    const [first, empty, highest] = chart.bars;
    expect(highest?.y).toBe(chart.plot.top);
    expect(empty?.height).toBe(0);
    expect(first?.height).toBeCloseTo((highest?.height ?? 0) / 2, 0);
  });

  it("keeps the bars inside the plot area", () => {
    const chart = dailyChart(days(Array.from({ length: 30 }, (_, index) => index)));
    const last = chart.bars.at(-1);
    expect(chart.bars[0]?.x).toBeGreaterThanOrEqual(chart.plot.left);
    expect((last?.x ?? 0) + (last?.width ?? 0)).toBeLessThanOrEqual(chart.plot.right);
  });
});

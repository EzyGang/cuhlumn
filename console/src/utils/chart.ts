/** The steps that an axis can use, before the power of ten. */
const STEPS = [1, 2, 5, 10] as const;

/**
 * Returns a round step for an axis from zero to `value` with about `target` steps.
 * The step is a whole number, because the chart shows counts.
 */
export const niceStep = (value: number, target = 4): number => {
  if (!(value > 0) || !(target > 0)) {
    return 1;
  }
  const raw = value / target;
  const power = 10 ** Math.floor(Math.log10(raw));
  const step = (STEPS.find((factor) => factor * power >= raw) ?? 10) * power;
  return Math.max(1, Math.round(step));
};

/** Returns the smallest multiple of the axis step that is at least `value`. */
export const niceMax = (value: number, target = 4): number => {
  const step = niceStep(value, target);
  return Math.max(step, Math.ceil(value / step) * step);
};

/** Returns the axis tick values from zero to the nice maximum, in ascending order. */
export const axisTicks = (value: number, target = 4): number[] => {
  const step = niceStep(value, target);
  const max = niceMax(value, target);
  return Array.from({ length: Math.round(max / step) + 1 }, (_, index) => index * step);
};

/**
 * Returns the indexes of the days that get a label below the chart.
 * The first and the last day always get a label.
 */
export const labelIndexes = (count: number, maxLabels: number): number[] => {
  if (count <= 0) {
    return [];
  }
  if (count === 1 || maxLabels < 2) {
    return [0];
  }
  const gaps = Math.min(count - 1, maxLabels - 1);
  const indexes = Array.from({ length: gaps + 1 }, (_, index) =>
    Math.round((index * (count - 1)) / gaps),
  );
  return [...new Set(indexes)];
};

/** Reads a `YYYY-MM-DD` day as midnight UTC. */
const parseDay = (date: string): Date => new Date(`${date}T00:00:00Z`);

/** Formats a `YYYY-MM-DD` day as a short label without the year, for example "10 Nov". */
export const shortDay = (date: string, locale?: string): string =>
  new Intl.DateTimeFormat(locale, { day: "numeric", month: "short", timeZone: "UTC" }).format(
    parseDay(date),
  );

/** Formats a `YYYY-MM-DD` day with the year, for example "10 Nov 2026". */
export const longDay = (date: string, locale?: string): string =>
  new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeZone: "UTC" }).format(parseDay(date));

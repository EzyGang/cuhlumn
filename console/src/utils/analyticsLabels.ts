import type { Browser } from "../types/api/Browser";
import type { Device } from "../types/api/Device";
import type { Period } from "../types/api/Period";
import type { SelectItem } from "../types/select";

export const PERIOD_LABELS: Readonly<Record<Period, string>> = {
  week: "Last 7 days",
  month: "Last 30 days",
  quarter: "Last 90 days",
  year: "Last 365 days",
};

export const PERIOD_ITEMS: readonly SelectItem<Period>[] = (
  ["week", "month", "quarter", "year"] as const
).map((value) => ({ value, label: PERIOD_LABELS[value] }));

export const DEVICE_LABELS: Readonly<Record<Device, string>> = {
  desktop: "Desktop",
  mobile: "Mobile",
  tablet: "Tablet",
};

export const BROWSER_LABELS: Readonly<Record<Browser, string>> = {
  chrome: "Chrome",
  safari: "Safari",
  firefox: "Firefox",
  edge: "Edge",
  opera: "Opera",
  samsung: "Samsung Internet",
  other: "Other",
};

/** The label of a referrer. A null host is a direct visit. */
export const referrerLabel = (host: string | null): string => host ?? "Direct";

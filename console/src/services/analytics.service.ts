import type { Period } from "../types/api/Period";
import type { SiteAnalytics } from "../types/api/SiteAnalytics";
import { requestJson, segment } from "./http";

export const getAnalytics = (host: string, period: Period): Promise<SiteAnalytics> =>
  requestJson<SiteAnalytics>("GET", `/api/sites/${segment(host)}/analytics?period=${period}`);

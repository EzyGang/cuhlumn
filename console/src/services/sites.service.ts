import type { Site } from "../types/api/Site";
import type { SiteInput } from "../types/api/SiteInput";
import { requestEmpty, requestJson, segment } from "./http";

export const listSites = (): Promise<Site[]> => requestJson<Site[]>("GET", "/api/sites");

export const getSite = (host: string): Promise<Site> =>
  requestJson<Site>("GET", `/api/sites/${segment(host)}`);

export const saveSite = (host: string, input: SiteInput): Promise<Site> =>
  requestJson<Site>("PUT", `/api/sites/${segment(host)}`, { json: input });

export const deleteSite = (host: string): Promise<void> =>
  requestEmpty("DELETE", `/api/sites/${segment(host)}`);

import type { LoadState } from "./load";

export interface SiteRow {
  host: string;
  title: string;
  aliasLabel: string;
  href: string;
}

export interface SitesModel {
  data: { sites: SiteRow[] };
  status: { load: LoadState };
}

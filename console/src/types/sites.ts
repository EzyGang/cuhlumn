import type { LoadState } from "./load";

export interface SiteRow {
  host: string;
  title: string;
  /** True for a website that another server serves. The console shows only its analytics. */
  external: boolean;
  /** The alias count of a blog, or a note for an external site. */
  detail: string;
  href: string;
}

export interface SitesModel {
  data: { sites: SiteRow[] };
  status: { load: LoadState };
}

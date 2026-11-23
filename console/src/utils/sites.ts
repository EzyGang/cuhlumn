import type { Site } from "../types/api/Site";
import type { SiteKind } from "../types/api/SiteKind";

type SiteRef = Pick<Site, "host" | "kind">;

/** Returns the kind of the site at `host`, or null when the list did not load yet or has no such site. */
export const siteKind = (sites: readonly SiteRef[] | null, host: string): SiteKind | null =>
  sites?.find((site) => site.host === host)?.kind ?? null;

/** Returns the address of the analytics page of a site. */
export const analyticsPath = (host: string): string =>
  `/sites/${encodeURIComponent(host)}/analytics`;

/** Returns the first page of a site: the posts of a blog, or the analytics of an external site. */
export const siteHomePath = (site: SiteRef): string =>
  site.kind === "external"
    ? analyticsPath(site.host)
    : `/sites/${encodeURIComponent(site.host)}/posts`;

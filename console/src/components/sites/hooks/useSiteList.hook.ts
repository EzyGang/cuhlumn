import { useSites } from "../../../hooks/useSites.hook";
import type { Site } from "../../../types/api/Site";
import type { SiteRow, SitesModel } from "../../../types/sites";
import { siteHomePath } from "../../../utils/sites";

const detail = (site: Site): string => {
  if (site.kind === "external") {
    return "Analytics only";
  }
  return site.aliases.length === 1 ? "1 alias" : `${site.aliases.length} aliases`;
};

const siteRow = (site: Site): SiteRow => ({
  host: site.host,
  title: site.title,
  external: site.kind === "external",
  detail: detail(site),
  href: siteHomePath(site),
});

export const useSiteList = (): SitesModel => {
  const { sites, load } = useSites();
  return {
    data: { sites: (sites ?? []).map(siteRow) },
    status: { load },
  };
};

import { useSites } from "../../../hooks/useSites.hook";
import type { SitesModel } from "../../../types/sites";

export const useSiteList = (): SitesModel => {
  const { sites, load } = useSites();
  return {
    data: {
      sites: (sites ?? []).map((site) => ({
        host: site.host,
        title: site.title,
        aliasLabel: site.aliases.length === 1 ? "1 alias" : `${site.aliases.length} aliases`,
        href: `/sites/${encodeURIComponent(site.host)}/posts`,
      })),
    },
    status: { load },
  };
};

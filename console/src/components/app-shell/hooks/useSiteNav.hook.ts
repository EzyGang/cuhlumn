import { useLocation } from "wouter-preact";
import { sitesStore } from "../../../stores/sites.store";
import type { SiteNavItem } from "../../../types/appShell";
import { siteKind } from "../../../utils/sites";

interface SiteNavModel {
  items: SiteNavItem[];
}

interface SectionRule {
  label: string;
  suffix: string;
  /** True for a section that an external site does not have. */
  blogOnly: boolean;
  matches: (path: string, base: string) => boolean;
}

const SECTIONS: readonly SectionRule[] = [
  {
    label: "Posts",
    suffix: "/posts",
    blogOnly: true,
    matches: (path, base): boolean =>
      path === `${base}/posts` || path.startsWith(`${base}/posts/`) || path === `${base}/new-post`,
  },
  {
    label: "Media",
    suffix: "/media",
    blogOnly: true,
    matches: (path, base): boolean => path === `${base}/media`,
  },
  {
    label: "Analytics",
    suffix: "/analytics",
    blogOnly: false,
    matches: (path, base): boolean => path === `${base}/analytics`,
  },
  {
    label: "Settings",
    suffix: "",
    blogOnly: true,
    matches: (path, base): boolean => path === base,
  },
];

/**
 * Lists the site sections and marks the one that matches the current location.
 * An external site has only the analytics section.
 */
export const useSiteNav = (host: string): SiteNavModel => {
  const [location] = useLocation();
  const base = `/sites/${encodeURIComponent(host)}`;
  const path = location.replace(/\/+$/, "");
  const external = siteKind(sitesStore.sites.value, host) === "external";
  return {
    items: SECTIONS.filter((section) => !(external && section.blogOnly)).map((section) => ({
      label: section.label,
      href: `${base}${section.suffix}`,
      current: section.matches(path, base) ? "page" : undefined,
    })),
  };
};

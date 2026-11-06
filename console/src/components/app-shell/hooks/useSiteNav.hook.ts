import { useLocation } from "wouter-preact";
import type { SiteNavItem } from "../../../types/appShell";

interface SiteNavModel {
  items: SiteNavItem[];
}

interface SectionRule {
  label: string;
  suffix: string;
  matches: (path: string, base: string) => boolean;
}

const SECTIONS: readonly SectionRule[] = [
  {
    label: "Posts",
    suffix: "/posts",
    matches: (path, base): boolean =>
      path === `${base}/posts` || path.startsWith(`${base}/posts/`) || path === `${base}/new-post`,
  },
  { label: "Media", suffix: "/media", matches: (path, base): boolean => path === `${base}/media` },
  { label: "Settings", suffix: "", matches: (path, base): boolean => path === base },
];

/** Lists the site sections and marks the one that matches the current location. */
export const useSiteNav = (host: string): SiteNavModel => {
  const [location] = useLocation();
  const base = `/sites/${encodeURIComponent(host)}`;
  const path = location.replace(/\/+$/, "");
  return {
    items: SECTIONS.map((section) => ({
      label: section.label,
      href: `${base}${section.suffix}`,
      current: section.matches(path, base) ? "page" : undefined,
    })),
  };
};

import { useLocation } from "wouter-preact";
import type { SiteNavItem } from "../../../types/appShell";

interface ServerNavModel {
  items: SiteNavItem[];
}

const PAGES: readonly { label: string; href: string }[] = [
  { label: "Fonts", href: "/fonts" },
  { label: "Server", href: "/settings" },
];

/** Lists the server pages and marks the one that matches the current location. */
export const useServerNav = (): ServerNavModel => {
  const [location] = useLocation();
  const path = location.replace(/\/+$/, "");
  return {
    items: PAGES.map((page) => ({
      label: page.label,
      href: page.href,
      current: path === page.href ? "page" : undefined,
    })),
  };
};

import type { JSX } from "preact";
import { Link } from "wouter-preact";
import type { SiteNavItem } from "../../../types/appShell";

interface SiteNavProps {
  items: SiteNavItem[];
}

export const SiteNavView = ({ items }: SiteNavProps): JSX.Element => (
  <nav aria-label="Site" class="mx-auto flex max-w-7xl gap-1 px-5">
    {items.map((item) => (
      <Link
        key={item.label}
        href={item.href}
        aria-current={item.current}
        class="border-b-2 border-transparent px-3 py-2.5 text-xs font-medium tracking-wide text-muted uppercase transition-colors duration-150 hover:text-white aria-[current=page]:border-white aria-[current=page]:text-white"
      >
        {item.label}
      </Link>
    ))}
  </nav>
);

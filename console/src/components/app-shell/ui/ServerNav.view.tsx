import type { JSX } from "preact";
import { Link } from "wouter-preact";
import type { SiteNavItem } from "../../../types/appShell";

interface ServerNavProps {
  items: SiteNavItem[];
}

export const ServerNavView = ({ items }: ServerNavProps): JSX.Element => (
  <nav aria-label="Server" class="ml-auto flex gap-1">
    {items.map((item) => (
      <Link
        key={item.label}
        href={item.href}
        aria-current={item.current}
        class="px-3 py-2 text-xs font-medium tracking-wide text-muted uppercase transition-colors duration-150 hover:text-white aria-[current=page]:text-white"
      >
        {item.label}
      </Link>
    ))}
  </nav>
);

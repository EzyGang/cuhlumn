import type { JSX } from "preact";
import { Link } from "wouter-preact";
import type { SiteRow } from "../../../types/sites";

interface SiteCardProps {
  site: SiteRow;
}

export const SiteCardView = ({ site }: SiteCardProps): JSX.Element => (
  <li>
    <Link
      href={site.href}
      class="flex h-full flex-col gap-2 rounded-sm border border-border bg-surface p-5 transition-colors duration-150 hover:border-hover-border hover:bg-card"
    >
      <span class="truncate text-lg font-semibold">{site.title}</span>
      <span class="truncate font-mono text-sm text-secondary">{site.host}</span>
      <span class="eyebrow">{site.aliasLabel}</span>
    </Link>
  </li>
);

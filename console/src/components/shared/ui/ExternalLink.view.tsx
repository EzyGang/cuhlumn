import type { ComponentChildren, JSX } from "preact";
import { ExternalLink as ExternalLinkIcon } from "lucide-preact";

interface ExternalLinkProps {
  href: string;
  children: ComponentChildren;
}

export const ExternalLink = ({ href, children }: ExternalLinkProps): JSX.Element => (
  <a
    href={href}
    target="_blank"
    rel="noreferrer"
    class="inline-flex min-w-0 items-center gap-1 text-sm text-secondary hover:text-white hover:underline"
  >
    <span class="truncate">{children}</span>
    <ExternalLinkIcon size={12} aria-hidden="true" class="shrink-0" />
  </a>
);

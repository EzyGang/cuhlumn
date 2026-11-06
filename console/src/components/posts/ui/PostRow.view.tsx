import type { JSX } from "preact";
import { Link } from "wouter-preact";
import { Badge } from "../../shared/ui/Badge.view";
import { ExternalLink } from "../../shared/ui/ExternalLink.view";
import type { PostRow } from "../../../types/posts";

interface PostRowProps {
  row: PostRow;
}

export const PostRowView = ({ row }: PostRowProps): JSX.Element => (
  <tr class="border-t border-border hover:bg-surface">
    <td class="px-4 py-3">
      <Link href={row.editHref} class="flex flex-col gap-0.5 hover:underline">
        <span class="font-medium">{row.title}</span>
        <span class="font-mono text-xs text-muted">/{row.slug}</span>
      </Link>
    </td>
    <td class="px-4 py-3">
      <Badge tone={row.statusTone}>{row.status}</Badge>
    </td>
    <td class="px-4 py-3">
      <Badge>{row.kind}</Badge>
    </td>
    <td class="px-4 py-3 whitespace-nowrap text-secondary">{row.updated}</td>
    <td class="px-4 py-3 text-right">
      {row.publicUrl === null ? null : <ExternalLink href={row.publicUrl}>View</ExternalLink>}
    </td>
  </tr>
);

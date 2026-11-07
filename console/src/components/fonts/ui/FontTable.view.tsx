import type { JSX } from "preact";
import { Trash2 } from "lucide-preact";
import { ConfirmDialog } from "../../shared/ui/ConfirmDialog.view";
import type { FontRow } from "../../../types/fonts";

interface FontTableProps {
  rows: FontRow[];
}

export const FontTable = ({ rows }: FontTableProps): JSX.Element => (
  <div class="overflow-x-auto border border-border">
    <table class="w-full text-left text-sm">
      <thead class="bg-surface text-xs tracking-wide text-muted uppercase">
        <tr>
          <th class="px-4 py-3 font-medium">Families</th>
          <th class="px-4 py-3 font-medium">Source</th>
          <th class="px-4 py-3 font-medium">Size</th>
          <th class="px-4 py-3 font-medium">Added</th>
          <th class="px-4 py-3 font-medium">
            <span class="sr-only">Actions</span>
          </th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr key={row.name} class="border-t border-border">
            <td class="px-4 py-3">
              <p>{row.families}</p>
              <p class="font-mono text-xs text-muted">{row.name}</p>
            </td>
            <td class="px-4 py-3 text-secondary">{row.source}</td>
            <td class="px-4 py-3 text-secondary">{row.size}</td>
            <td class="px-4 py-3 text-secondary">{row.created}</td>
            <td class="px-4 py-3 text-right">
              <ConfirmDialog
                trigger={<Trash2 size={14} aria-hidden="true" />}
                triggerLabel={`Delete ${row.name}`}
                title="Delete this font?"
                description="Share images that need its characters fall back to other fonts. You cannot undo this."
                confirmLabel="Delete font"
                onConfirm={row.remove}
              />
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  </div>
);

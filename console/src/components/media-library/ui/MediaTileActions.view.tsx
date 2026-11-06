import type { JSX } from "preact";
import { Copy, Trash2 } from "lucide-preact";
import { Button } from "../../shared/ui/Button.view";
import { ConfirmDialog } from "../../shared/ui/ConfirmDialog.view";
import type { MediaTile } from "../../../types/media";

interface MediaTileActionsProps {
  tile: MediaTile;
}

export const MediaTileActions = ({ tile }: MediaTileActionsProps): JSX.Element => (
  <div class="mt-auto flex gap-2 border-t border-border p-3">
    <Button variant="ghost" onClick={tile.copy} class="flex-1">
      <Copy size={14} aria-hidden="true" />
      Copy
    </Button>
    <ConfirmDialog
      trigger={<Trash2 size={14} aria-hidden="true" />}
      triggerLabel={`Delete ${tile.name}`}
      title="Delete this image?"
      description="Pages that use the address show a broken image. You cannot undo this."
      confirmLabel="Delete image"
      onConfirm={tile.remove}
    />
  </div>
);

import type { JSX } from "preact";
import type { MediaTile } from "../../../types/media";
import { MediaTileActions } from "./MediaTileActions.view";

interface MediaTileProps {
  tile: MediaTile;
}

export const MediaTileView = ({ tile }: MediaTileProps): JSX.Element => (
  <li class="flex min-w-0 flex-col rounded-sm border border-border bg-surface">
    <a
      href={tile.url}
      target="_blank"
      rel="noreferrer"
      aria-label={`Open ${tile.name}`}
      class="block overflow-hidden bg-card"
    >
      <img src={tile.url} alt="" loading="lazy" class="aspect-square w-full object-contain" />
    </a>
    <div class="flex min-w-0 flex-col gap-1 p-3">
      <p class="truncate font-mono text-xs" title={tile.name}>
        {tile.name}
      </p>
      <p class="text-xs text-muted">{tile.details}</p>
      <p class="text-xs text-muted">{tile.created}</p>
    </div>
    <MediaTileActions tile={tile} />
  </li>
);

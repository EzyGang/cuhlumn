import type { JSX } from "preact";
import type { MediaTile } from "../../../types/media";
import { MediaTileView } from "./MediaTile.view";

interface MediaGridProps {
  tiles: MediaTile[];
}

export const MediaGridView = ({ tiles }: MediaGridProps): JSX.Element => (
  <ul class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
    {tiles.map((tile) => (
      <MediaTileView key={tile.name} tile={tile} />
    ))}
  </ul>
);

import type { LoadState } from "./load";

export interface MediaTile {
  name: string;
  url: string;
  details: string;
  created: string;
  copy: () => void;
  remove: () => void;
}

export interface MediaModel {
  data: { tiles: MediaTile[]; dragging: boolean; dropText: string; accept: string };
  status: { load: LoadState; uploading: boolean };
  actions: {
    pickFiles: (event: Event) => void;
    dragOver: (event: DragEvent) => void;
    dragLeave: () => void;
    drop: (event: DragEvent) => void;
  };
}

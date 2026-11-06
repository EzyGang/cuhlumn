import type { JSX } from "preact";
import { Upload } from "lucide-preact";
import type { MediaModel } from "../../../types/media";

export const DropZoneView = ({ data, status, actions }: MediaModel): JSX.Element => (
  <div
    data-dragging={data.dragging}
    class="flex flex-col items-center justify-center gap-3 border border-dashed border-hover-border bg-surface p-8 text-center transition-colors duration-150 data-[dragging=true]:border-white data-[dragging=true]:bg-hover"
    onDragOver={actions.dragOver}
    onDragLeave={actions.dragLeave}
    onDrop={actions.drop}
  >
    <Upload size={20} aria-hidden="true" class="text-muted" />
    <p class="text-sm text-secondary" role="status">
      {data.dropText}
    </p>
    <label class="inline-flex cursor-pointer items-center gap-2 border border-border px-4 py-3 text-xs font-medium tracking-wide uppercase transition-colors duration-150 focus-within:outline-2 focus-within:outline-offset-3 focus-within:outline-white hover:border-hover-border hover:bg-hover">
      Choose files
      <input
        type="file"
        multiple
        accept={data.accept}
        class="sr-only"
        onChange={actions.pickFiles}
        disabled={status.uploading}
      />
    </label>
  </div>
);

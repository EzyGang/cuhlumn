import type { JSX } from "preact";
import { LoadGate } from "../../shared/ui/LoadGate.view";
import { PageHeader } from "../../shared/ui/PageHeader.view";
import type { MediaModel } from "../../../types/media";
import { DropZoneView } from "./DropZone.view";
import { MediaGridView } from "./MediaGrid.view";

export const MediaView = ({ data, status, actions }: MediaModel): JSX.Element => (
  <section class="flex flex-col gap-6">
    <PageHeader eyebrow="Media" title="Media library" />
    <DropZoneView data={data} status={status} actions={actions} />
    <LoadGate
      state={status.load}
      loadingText="Loading the media library"
      missingText="This site does not exist."
    >
      {data.tiles.length === 0 ? (
        <p class="panel text-sm text-secondary">There are no images yet.</p>
      ) : (
        <MediaGridView tiles={data.tiles} />
      )}
    </LoadGate>
  </section>
);

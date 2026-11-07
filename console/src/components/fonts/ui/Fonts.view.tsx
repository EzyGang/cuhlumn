import type { JSX } from "preact";
import { LoadGate } from "../../shared/ui/LoadGate.view";
import { PageHeader } from "../../shared/ui/PageHeader.view";
import type { FontsModel } from "../../../types/fonts";
import { FontSources } from "./FontSources.view";
import { FontTable } from "./FontTable.view";

export const FontsView = ({ data, status, actions }: FontsModel): JSX.Element => (
  <section class="flex flex-col gap-6">
    <PageHeader
      eyebrow="Server"
      title="Font library"
      below={
        <p class="max-w-2xl text-sm text-secondary">
          Share images use these fonts for the characters that Inter does not have, such as Japanese
          or Arabic letters.
        </p>
      }
    />
    <FontSources data={data} status={status} actions={actions} />
    <LoadGate
      state={status.load}
      loadingText="Loading the font library"
      missingText="The font library is missing."
    >
      {data.rows.length === 0 ? (
        <p class="panel text-sm text-secondary">There are no fonts yet.</p>
      ) : (
        <FontTable rows={data.rows} />
      )}
    </LoadGate>
  </section>
);

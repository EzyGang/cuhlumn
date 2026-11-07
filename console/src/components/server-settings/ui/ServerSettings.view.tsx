import type { JSX } from "preact";
import { LoadGate } from "../../shared/ui/LoadGate.view";
import { PageHeader } from "../../shared/ui/PageHeader.view";
import type { ServerSettingsModel } from "../../../types/serverSettings";
import { AddressSection } from "./AddressSection.view";
import { TokenSection } from "./TokenSection.view";

export const ServerSettingsView = ({ data, status }: ServerSettingsModel): JSX.Element => (
  <section class="flex max-w-3xl flex-col gap-6">
    <PageHeader eyebrow="Server" title="Server settings" />
    <LoadGate
      state={status.load}
      loadingText="Loading the server settings"
      missingText="The server has no settings."
    >
      <AddressSection {...data.address} />
      <TokenSection {...data.rotation} />
    </LoadGate>
  </section>
);

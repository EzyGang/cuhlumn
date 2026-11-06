import type { JSX } from "preact";
import { Plus } from "lucide-preact";
import { LinkButton } from "../../shared/ui/LinkButton.view";
import { LoadGate } from "../../shared/ui/LoadGate.view";
import { PageHeader } from "../../shared/ui/PageHeader.view";
import type { SitesModel } from "../../../types/sites";
import { SiteCardView } from "./SiteCard.view";

export const SiteListView = ({ data, status }: SitesModel): JSX.Element => (
  <section class="flex flex-col gap-6">
    <PageHeader eyebrow="Sites" title="Your blogs">
      <LinkButton href="/new-site" variant="primary">
        <Plus size={14} aria-hidden="true" />
        New site
      </LinkButton>
    </PageHeader>
    <LoadGate
      state={status.load}
      loadingText="Loading the sites"
      missingText="The site list is missing."
    >
      {data.sites.length === 0 ? (
        <p class="panel text-sm text-secondary">There are no sites yet. Create the first site.</p>
      ) : (
        <ul class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {data.sites.map((site) => (
            <SiteCardView key={site.host} site={site} />
          ))}
        </ul>
      )}
    </LoadGate>
  </section>
);

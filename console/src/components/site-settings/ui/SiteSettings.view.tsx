import type { JSX } from "preact";
import { LoadGate } from "../../shared/ui/LoadGate.view";
import { useSiteSettingsContext } from "../context/SiteSettingsContext";
import { AnnouncementSection } from "./announcement/AnnouncementSection.view";
import { AppearanceSection } from "./appearance/AppearanceSection.view";
import { DangerZone } from "./DangerZone.view";
import { DomainSection } from "./DomainSection.view";
import { GeneralSection } from "./GeneralSection.view";
import { SiteSettingsHeader } from "./SiteSettingsHeader.view";

export const SiteSettingsView = (): JSX.Element => {
  const { data, status, actions } = useSiteSettingsContext();
  return (
    <LoadGate
      state={status.load}
      loadingText="Loading the site"
      missingText="This site does not exist."
    >
      <form class="flex flex-col gap-6" onSubmit={actions.save} noValidate>
        <SiteSettingsHeader />
        <div class="grid gap-6 lg:grid-cols-2">
          <div class="flex min-w-0 flex-col gap-6">
            <DomainSection />
            <GeneralSection />
            <AnnouncementSection />
          </div>
          <div class="flex min-w-0 flex-col gap-6">
            <AppearanceSection />
            {data.mode === "edit" ? <DangerZone /> : null}
          </div>
        </div>
      </form>
    </LoadGate>
  );
};

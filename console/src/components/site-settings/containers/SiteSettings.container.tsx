import type { JSX } from "preact";
import { SiteSettingsProvider } from "../context/SiteSettingsContext";
import { useSiteSettings } from "../hooks/useSiteSettings.hook";
import { SiteSettingsView } from "../ui/SiteSettings.view";

export const SiteSettingsContainer = (): JSX.Element => (
  <SiteSettingsProvider value={useSiteSettings()}>
    <SiteSettingsView />
  </SiteSettingsProvider>
);

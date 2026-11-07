import type { JSX } from "preact";
import { useServerSettings } from "../hooks/useServerSettings.hook";
import { ServerSettingsView } from "../ui/ServerSettings.view";

export const ServerSettingsContainer = (): JSX.Element => (
  <ServerSettingsView {...useServerSettings()} />
);

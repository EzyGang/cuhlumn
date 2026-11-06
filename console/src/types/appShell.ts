import type { ComponentChildren } from "preact";
import type { SelectItem } from "./select";

export interface SiteNavItem {
  label: string;
  href: string;
  current: "page" | undefined;
}

export interface AppShellModel {
  data: { hosts: SelectItem<string>[]; host: string | null };
  status: { loggingOut: boolean };
  actions: { switchSite: (host: string) => void; logout: () => void };
}

export interface AppShellProps extends AppShellModel {
  children: ComponentChildren;
}

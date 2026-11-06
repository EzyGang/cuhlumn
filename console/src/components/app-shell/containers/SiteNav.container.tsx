import type { JSX } from "preact";
import { useSiteNav } from "../hooks/useSiteNav.hook";
import { SiteNavView } from "../ui/SiteNav.view";

interface SiteNavContainerProps {
  host: string;
}

export const SiteNavContainer = ({ host }: SiteNavContainerProps): JSX.Element => (
  <SiteNavView {...useSiteNav(host)} />
);

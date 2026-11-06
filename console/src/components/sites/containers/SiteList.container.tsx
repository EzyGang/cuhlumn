import type { JSX } from "preact";
import { useSiteList } from "../hooks/useSiteList.hook";
import { SiteListView } from "../ui/SiteList.view";

export const SiteListContainer = (): JSX.Element => <SiteListView {...useSiteList()} />;

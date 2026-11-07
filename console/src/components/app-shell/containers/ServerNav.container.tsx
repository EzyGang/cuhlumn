import type { JSX } from "preact";
import { useServerNav } from "../hooks/useServerNav.hook";
import { ServerNavView } from "../ui/ServerNav.view";

export const ServerNavContainer = (): JSX.Element => <ServerNavView {...useServerNav()} />;

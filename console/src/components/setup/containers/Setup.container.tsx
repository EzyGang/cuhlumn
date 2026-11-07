import type { JSX } from "preact";
import { useSetup } from "../hooks/useSetup.hook";
import { SetupView } from "../ui/Setup.view";

export const SetupContainer = (): JSX.Element => <SetupView {...useSetup()} />;

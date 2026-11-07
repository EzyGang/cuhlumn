import type { JSX } from "preact";
import { useFonts } from "../hooks/useFonts.hook";
import { FontsView } from "../ui/Fonts.view";

export const FontsContainer = (): JSX.Element => <FontsView {...useFonts()} />;

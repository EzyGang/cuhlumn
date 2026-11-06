import type { ComponentChildren, JSX } from "preact";
import { useAppShell } from "../hooks/useAppShell.hook";
import { AppShellView } from "../ui/AppShell.view";

interface AppShellContainerProps {
  children: ComponentChildren;
}

export const AppShellContainer = ({ children }: AppShellContainerProps): JSX.Element => (
  <AppShellView {...useAppShell()}>{children}</AppShellView>
);

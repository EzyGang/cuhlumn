import type { ComponentChildren, JSX } from "preact";

interface AppearanceGroupProps {
  title: string;
  children: ComponentChildren;
}

export const AppearanceGroup = ({ title, children }: AppearanceGroupProps): JSX.Element => (
  <div class="flex flex-col gap-5 border-t border-border pt-5">
    <h3 class="text-xs font-semibold tracking-wide text-white uppercase">{title}</h3>
    {children}
  </div>
);

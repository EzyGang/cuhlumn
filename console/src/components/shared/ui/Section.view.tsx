import type { ComponentChildren, JSX } from "preact";

interface SectionProps {
  title: string;
  description?: string;
  children: ComponentChildren;
}

export const Section = ({ title, description, children }: SectionProps): JSX.Element => (
  <section class="flex flex-col gap-5 rounded-sm border border-border bg-surface p-5">
    <header class="flex flex-col gap-1">
      <h2 class="text-sm font-semibold tracking-wide uppercase">{title}</h2>
      {description === undefined ? null : <p class="text-sm text-muted">{description}</p>}
    </header>
    {children}
  </section>
);

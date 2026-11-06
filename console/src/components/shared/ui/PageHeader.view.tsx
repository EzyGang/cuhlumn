import type { ComponentChildren, JSX } from "preact";

interface PageHeaderProps {
  eyebrow: ComponentChildren;
  title: string;
  children?: ComponentChildren;
  below?: ComponentChildren;
}

export const PageHeader = ({ eyebrow, title, children, below }: PageHeaderProps): JSX.Element => (
  <header class="flex flex-wrap items-end justify-between gap-4">
    <div class="flex min-w-0 flex-col gap-2">
      <div class="eyebrow flex items-center gap-3">{eyebrow}</div>
      <h1 class="page-title truncate">{title}</h1>
      {below}
    </div>
    {children === undefined ? null : <div class="flex flex-wrap gap-3">{children}</div>}
  </header>
);

import type { JSX } from "preact";
import type { TocItem } from "../../../types/postEditor";

interface TocProps {
  items: TocItem[];
}

export const TocView = ({ items }: TocProps): JSX.Element | null =>
  items.length === 0 ? null : (
    <nav aria-label="Table of contents" class="flex flex-col gap-1 border-l border-border pl-3">
      <p class="eyebrow">Contents</p>
      <ol class="flex flex-col gap-1 text-sm text-secondary">
        {items.map((item) => (
          <li key={item.id} class={item.indent}>
            {item.label}
          </li>
        ))}
      </ol>
    </nav>
  );

import type { JSX } from "preact";
import type { MediaChoice } from "../../../../types/siteSettings";

interface MediaChoiceProps {
  item: MediaChoice;
}

export const MediaChoiceView = ({ item }: MediaChoiceProps): JSX.Element => (
  <li>
    <button
      type="button"
      onClick={item.pick}
      class="flex w-full flex-col gap-2 border border-border bg-surface p-2 text-left transition-colors duration-150 hover:border-hover-border hover:bg-hover"
    >
      <img
        src={item.url}
        alt=""
        class="aspect-square w-full bg-card object-contain"
        loading="lazy"
      />
      <span class="truncate font-mono text-xs text-secondary">{item.name}</span>
    </button>
  </li>
);

import type { JSX } from "preact";
import { X } from "lucide-preact";
import type { ChipValue } from "../../../types/chips";

interface ChipListProps {
  label: string;
  chips: ChipValue[];
}

export const ChipList = ({ label, chips }: ChipListProps): JSX.Element | null =>
  chips.length === 0 ? null : (
    <ul class="flex flex-wrap gap-2" aria-label={`${label} list`}>
      {chips.map((chip) => (
        <li
          key={chip.label}
          data-invalid={chip.invalid}
          class="inline-flex items-center gap-1 border border-border bg-subtle py-1 pr-1 pl-2.5 text-sm data-[invalid=true]:border-error"
        >
          {chip.label}
          <button
            type="button"
            class="p-0.5 text-muted hover:text-white"
            aria-label={`Remove ${chip.label}`}
            onClick={chip.remove}
          >
            <X size={12} aria-hidden="true" />
          </button>
        </li>
      ))}
    </ul>
  );

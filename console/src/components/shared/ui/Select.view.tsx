import type { JSX } from "preact";
import { Select as BaseSelect } from "@base-ui/react/select";
import { Check, ChevronsUpDown } from "lucide-preact";
import type { SelectItem } from "../../../types/select";

interface SelectProps<T extends string> {
  value: T | null;
  items: readonly SelectItem<T>[];
  onValueChange: (value: T) => void;
  label?: string;
  ariaLabel?: string;
  placeholder?: string;
  class?: string;
}

interface SelectPopupProps<T extends string> {
  items: readonly SelectItem<T>[];
}

const SelectPopup = <T extends string>({ items }: SelectPopupProps<T>): JSX.Element => (
  <BaseSelect.Portal>
    <BaseSelect.Positioner
      className="z-50 outline-hidden"
      sideOffset={4}
      alignItemWithTrigger={false}
    >
      <BaseSelect.Popup className="min-w-[var(--anchor-width)] border border-hover-border bg-card py-1 outline-hidden">
        <BaseSelect.List className="max-h-[var(--available-height)] overflow-y-auto">
          {items.map((item) => (
            <BaseSelect.Item
              key={item.value}
              value={item.value}
              className="grid cursor-default grid-cols-[1rem_1fr] items-center gap-2 py-2 pr-4 pl-2.5 text-sm text-secondary outline-hidden select-none data-[highlighted]:bg-hover data-[highlighted]:text-white"
            >
              <BaseSelect.ItemIndicator className="col-start-1">
                <Check size={14} aria-hidden="true" />
              </BaseSelect.ItemIndicator>
              <BaseSelect.ItemText className="col-start-2">{item.label}</BaseSelect.ItemText>
            </BaseSelect.Item>
          ))}
        </BaseSelect.List>
      </BaseSelect.Popup>
    </BaseSelect.Positioner>
  </BaseSelect.Portal>
);

interface SelectControlProps {
  label: string | undefined;
  ariaLabel: string | undefined;
  placeholder: string | undefined;
  extra: string | undefined;
}

const SelectControl = ({
  label,
  ariaLabel,
  placeholder,
  extra,
}: SelectControlProps): JSX.Element => (
  <div class={`flex min-w-0 flex-col gap-1.5 ${extra ?? ""}`}>
    {label === undefined ? null : (
      <BaseSelect.Label className="text-xs font-medium tracking-wide text-secondary uppercase">
        {label}
      </BaseSelect.Label>
    )}
    <BaseSelect.Trigger
      aria-label={ariaLabel}
      className="flex min-w-36 items-center justify-between gap-3 border border-border bg-surface px-3 py-2.5 text-left text-sm text-white transition-colors duration-150 hover:border-hover-border data-[popup-open]:border-hover-border"
    >
      <BaseSelect.Value
        className="truncate data-[placeholder]:text-muted"
        placeholder={placeholder}
      />
      <BaseSelect.Icon className="text-muted">
        <ChevronsUpDown size={14} aria-hidden="true" />
      </BaseSelect.Icon>
    </BaseSelect.Trigger>
  </div>
);

export const Select = <T extends string>({
  value,
  items,
  onValueChange,
  label,
  ariaLabel,
  placeholder,
  class: extra,
}: SelectProps<T>): JSX.Element => (
  <BaseSelect.Root
    items={items}
    value={value}
    onValueChange={(next: T | null): void => (next === null ? undefined : onValueChange(next))}
  >
    <SelectControl label={label} ariaLabel={ariaLabel} placeholder={placeholder} extra={extra} />
    <SelectPopup items={items} />
  </BaseSelect.Root>
);

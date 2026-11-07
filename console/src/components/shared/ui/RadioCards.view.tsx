import type { JSX } from "preact";
import { useId } from "preact/hooks";
import { Radio } from "@base-ui/react/radio";
import { RadioGroup } from "@base-ui/react/radio-group";
import type { RadioCardItem } from "../../../types/radioCards";

type Columns = 2 | 3;

const COLUMNS: Record<Columns, string> = {
  2: "sm:grid-cols-2",
  3: "sm:grid-cols-3",
};

interface RadioCardsProps<T extends string> {
  label: string;
  value: T;
  items: readonly RadioCardItem<T>[];
  onValueChange: (value: T) => void;
  columns?: Columns;
}

interface RadioCardProps<T extends string> {
  item: RadioCardItem<T>;
  id: string;
}

const RadioCard = <T extends string>({ item, id }: RadioCardProps<T>): JSX.Element => (
  <label class="flex cursor-pointer items-start gap-3 border border-border bg-background p-3 transition-colors duration-150 hover:border-hover-border hover:bg-hover has-[[data-checked]]:border-white">
    <Radio.Root
      value={item.value}
      aria-labelledby={`${id}-label`}
      aria-describedby={`${id}-description`}
      className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border border-hover-border data-[checked]:border-white"
    >
      <Radio.Indicator className="size-2 rounded-full bg-white" />
    </Radio.Root>
    <span class="flex min-w-0 flex-col gap-0.5">
      <span id={`${id}-label`} class="text-sm text-white">
        {item.label}
      </span>
      <span id={`${id}-description`} class="text-xs text-muted">
        {item.description}
      </span>
    </span>
  </label>
);

export const RadioCards = <T extends string>({
  label,
  value,
  items,
  onValueChange,
  columns = 2,
}: RadioCardsProps<T>): JSX.Element => {
  const id = useId();
  return (
    <div class="flex min-w-0 flex-col gap-1.5">
      <span id={id} class="text-xs font-medium tracking-wide text-secondary uppercase">
        {label}
      </span>
      <RadioGroup
        aria-labelledby={id}
        value={value}
        onValueChange={(next: unknown): void => onValueChange(next as T)}
        className={`grid gap-2 ${COLUMNS[columns]}`}
      >
        {items.map((item) => (
          <RadioCard key={item.value} item={item} id={`${id}-${item.value}`} />
        ))}
      </RadioGroup>
    </div>
  );
};

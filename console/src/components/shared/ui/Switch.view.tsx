import type { JSX } from "preact";
import { Switch as BaseSwitch } from "@base-ui/react/switch";

interface SwitchProps {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
}

export const Switch = ({ checked, onCheckedChange }: SwitchProps): JSX.Element => (
  <BaseSwitch.Root
    checked={checked}
    onCheckedChange={onCheckedChange}
    className="inline-flex h-6 w-11 shrink-0 items-center border border-hover-border bg-background p-0.5 data-[checked]:border-white data-[checked]:bg-white"
  >
    <BaseSwitch.Thumb className="size-4 bg-white transition-transform data-[checked]:translate-x-5 data-[checked]:bg-background" />
  </BaseSwitch.Root>
);

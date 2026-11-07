import type { JSX } from "preact";
import { Input as BaseInput } from "@base-ui/react/input";

interface InputProps {
  value: string;
  onValueChange: (value: string) => void;
  type?: "text" | "password" | "url" | "search" | "color" | "datetime-local" | undefined;
  placeholder?: string | undefined;
  autoComplete?: string | undefined;
  disabled?: boolean | undefined;
  readOnly?: boolean | undefined;
  required?: boolean | undefined;
  label?: string | undefined;
  invalid?: boolean | undefined;
  spellCheck?: boolean | undefined;
  class?: string | undefined;
  onKeyDown?: (event: KeyboardEvent) => void;
  onBlur?: () => void;
}

export const Input = ({
  value,
  onValueChange,
  type = "text",
  label,
  invalid = false,
  class: extra,
  ...rest
}: InputProps): JSX.Element => (
  <BaseInput
    value={value}
    onValueChange={onValueChange}
    type={type}
    aria-label={label}
    aria-invalid={invalid || undefined}
    className={`data-[invalid]:border-error ${invalid ? "border-error" : ""} ${extra ?? ""}`}
    {...rest}
  />
);

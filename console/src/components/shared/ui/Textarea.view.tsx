import type { JSX } from "preact";
import { Field as BaseField } from "@base-ui/react/field";

interface TextareaProps {
  value: string;
  onValueChange: (value: string) => void;
  rows?: number | undefined;
  placeholder?: string | undefined;
  label?: string | undefined;
  mono?: boolean | undefined;
  class?: string | undefined;
}

export const Textarea = ({
  value,
  onValueChange,
  rows = 4,
  placeholder,
  label,
  mono = false,
  class: extra,
}: TextareaProps): JSX.Element => (
  <BaseField.Control
    value={value}
    onValueChange={onValueChange}
    placeholder={placeholder}
    aria-label={label}
    render={<textarea rows={rows} spellcheck={!mono} />}
    className={`data-[invalid]:border-error ${mono ? "font-mono text-sm sm:text-sm" : ""} ${extra ?? ""}`}
  />
);

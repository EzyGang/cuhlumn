import type { JSX } from "preact";
import { Field } from "./Field.view";
import { Input } from "./Input.view";
import { Textarea } from "./Textarea.view";

interface TextFieldProps {
  label: string;
  value: string;
  onValueChange: (value: string) => void;
  error?: string | undefined;
  description?: string | undefined;
  placeholder?: string | undefined;
  type?: "text" | "url" | "datetime-local" | undefined;
  disabled?: boolean | undefined;
  readOnly?: boolean | undefined;
  multiline?: number | undefined;
  mono?: boolean | undefined;
  class?: string | undefined;
}

export const TextField = ({
  label,
  error,
  description,
  multiline,
  mono,
  class: extra,
  ...control
}: TextFieldProps): JSX.Element => (
  <Field label={label} error={error} description={description} class={extra}>
    {multiline === undefined ? (
      <Input {...control} class={mono === true ? "font-mono" : ""} />
    ) : (
      <Textarea
        value={control.value}
        onValueChange={control.onValueChange}
        placeholder={control.placeholder}
        rows={multiline}
        mono={mono}
      />
    )}
  </Field>
);

import type { ComponentChildren, JSX } from "preact";
import { Field as BaseField } from "@base-ui/react/field";

interface FieldProps {
  label: string;
  children: ComponentChildren;
  error?: string | undefined;
  description?: string | undefined;
  class?: string | undefined;
}

export const Field = ({
  label,
  children,
  error,
  description,
  class: extra,
}: FieldProps): JSX.Element => (
  <BaseField.Root
    invalid={error !== undefined}
    className={`flex min-w-0 flex-col gap-1.5 ${extra ?? ""}`}
  >
    <BaseField.Label className="text-xs font-medium tracking-wide text-secondary uppercase">
      {label}
    </BaseField.Label>
    {children}
    {description === undefined ? null : (
      <BaseField.Description className="text-xs text-muted">{description}</BaseField.Description>
    )}
    {error === undefined ? null : (
      <BaseField.Error match className="text-xs text-error" role="alert">
        {error}
      </BaseField.Error>
    )}
  </BaseField.Root>
);

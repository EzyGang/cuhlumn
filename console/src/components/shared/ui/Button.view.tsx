import type { ComponentChildren, JSX } from "preact";
import { Button as BaseButton } from "@base-ui/react/button";
import { buttonClasses } from "./buttonClasses";
import type { ButtonVariant } from "./buttonClasses";

interface ButtonProps {
  children: ComponentChildren;
  variant?: ButtonVariant;
  type?: "button" | "submit";
  disabled?: boolean;
  label?: string;
  class?: string;
  onClick?: () => void;
}

export const Button = ({
  children,
  variant = "secondary",
  type = "button",
  disabled = false,
  label,
  class: extra,
  onClick,
}: ButtonProps): JSX.Element => (
  <BaseButton
    type={type}
    disabled={disabled}
    aria-label={label}
    onClick={onClick}
    className={buttonClasses(variant, extra)}
  >
    {children}
  </BaseButton>
);

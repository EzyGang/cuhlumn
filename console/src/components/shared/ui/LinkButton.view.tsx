import type { ComponentChildren, JSX } from "preact";
import { Link } from "wouter-preact";
import { buttonClasses } from "./buttonClasses";
import type { ButtonVariant } from "./buttonClasses";

interface LinkButtonProps {
  href: string;
  children: ComponentChildren;
  variant?: ButtonVariant;
  class?: string;
}

export const LinkButton = ({
  href,
  children,
  variant = "secondary",
  class: extra,
}: LinkButtonProps): JSX.Element => (
  <Link href={href} class={buttonClasses(variant, extra)}>
    {children}
  </Link>
);

export type ButtonVariant = "primary" | "secondary" | "danger" | "ghost";

const BASE =
  "inline-flex shrink-0 items-center justify-center gap-2 rounded-none text-xs font-medium tracking-wide uppercase transition-colors duration-150 select-none";

const VARIANTS: Record<ButtonVariant, string> = {
  primary: "border border-white bg-white px-4 py-3 text-background hover:opacity-85",
  secondary:
    "border border-border bg-transparent px-4 py-3 text-white hover:border-hover-border hover:bg-hover",
  danger:
    "border border-error/60 bg-transparent px-4 py-3 text-white hover:border-error hover:bg-error/20",
  ghost: "border border-transparent px-3 py-2 text-secondary hover:bg-hover hover:text-white",
};

export const buttonClasses = (variant: ButtonVariant, extra?: string): string =>
  `${BASE} ${VARIANTS[variant]} ${extra ?? ""}`;

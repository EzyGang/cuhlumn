import type { JSX } from "preact";

export type BadgeTone = "neutral" | "success" | "warning";

const TONES: Record<BadgeTone, string> = {
  neutral: "text-secondary",
  success: "text-success",
  warning: "text-orange",
};

interface BadgeProps {
  children: string;
  tone?: BadgeTone;
}

export const Badge = ({ children, tone = "neutral" }: BadgeProps): JSX.Element => (
  <span
    class={`inline-flex items-center border border-border bg-subtle px-2 py-0.5 text-[0.6875rem] font-medium tracking-wide uppercase ${TONES[tone]}`}
  >
    {children}
  </span>
);

import type { ElementType, ReactNode } from "react";
import { forwardRef } from "react";
import { cn } from "../../lib/utils";

interface NbSectionProps {
  variant?: "default" | "lg" | "sm" | "dark" | "sm-dark";
  ariaLabel?: string;
  as?: ElementType;
  children: ReactNode;
  className?: string;
}

const variantClasses: Record<string, string> = {
  default: "nb-section",
  lg: "nb-section nb-section--lg",
  sm: "nb-section nb-section--sm",
  dark: "nb-section nb-section--dark",
  "sm-dark": "nb-section nb-section--sm nb-section--dark",
};

export const NbSection = forwardRef<HTMLElement, NbSectionProps>(function NbSection(
  { variant = "default", ariaLabel, as: Tag = "section", children, className },
  ref,
) {
  return (
    <Tag ref={ref} className={cn(variantClasses[variant], className)} aria-label={ariaLabel}>
      <div className="nb-inner">{children}</div>
    </Tag>
  );
});

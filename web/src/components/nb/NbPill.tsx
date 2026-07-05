import { cn } from "../../lib/utils";

interface NbPillProps {
  children: React.ReactNode;
  variant?: "default" | "amber" | "green" | "red";
  className?: string;
}

export function NbPill({ children, variant = "default", className }: NbPillProps) {
  return (
    <span
      className={cn(
        "nb-pill",
        variant === "amber" && "nb-pill--amber",
        variant === "green" && "nb-pill--green",
        variant === "red" && "nb-pill--red",
        className,
      )}
    >
      {children}
    </span>
  );
}

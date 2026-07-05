import { cn } from "../../lib/utils";

interface NbFrameProps {
  children: React.ReactNode;
  label: string;
  variant?: "default" | "dark";
  className?: string;
}

export function NbFrame({ children, label, variant = "default", className }: NbFrameProps) {
  return (
    <div
      className={cn("nb-frame", variant === "dark" && "nb-frame--dark", className)}
      data-frame-label={label}
    >
      {children}
    </div>
  );
}

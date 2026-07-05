import { cn } from "../../lib/utils";

interface NbIconBoxProps {
  children: React.ReactNode;
  size?: "md" | "sm";
  className?: string;
}

export function NbIconBox({ children, size = "md", className }: NbIconBoxProps) {
  return (
    <span className={cn("nb-icon-box", size === "sm" && "nb-icon-box--sm", className)}>
      {children}
    </span>
  );
}

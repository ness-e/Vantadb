import { cn } from "../../lib/utils";

interface NbCardProps {
  children: React.ReactNode;
  variant?: "default" | "amber" | "strong" | "inset" | "offset" | "offset-amber";
  className?: string;
  as?: "div" | "article" | "li";
  tabIndex?: number;
  onClick?: () => void;
}

export function NbCard({
  children,
  variant = "default",
  className,
  as: Tag = "div",
  tabIndex,
  onClick,
}: NbCardProps) {
  return (
    <Tag
      className={cn(
        "nb-card",
        variant === "amber" && "nb-card--amber",
        variant === "strong" && "nb-card--strong",
        variant === "inset" && "nb-card--inset",
        variant === "offset" && "nb-card--offset",
        variant === "offset-amber" && "nb-card--offset-amber",
        className,
      )}
      tabIndex={tabIndex}
      onClick={onClick}
    >
      {children}
    </Tag>
  );
}

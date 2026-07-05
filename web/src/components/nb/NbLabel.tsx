import { cn } from "../../lib/utils";

interface NbLabelProps {
  children: React.ReactNode;
  variant?: "steel" | "amber" | "muted";
  className?: string;
  as?: "span" | "div" | "label";
}

export function NbLabel({
  children,
  variant = "steel",
  className,
  as: Tag = "span",
}: NbLabelProps) {
  return (
    <Tag
      className={cn(
        "nb-label",
        variant === "amber" && "nb-label--amber",
        variant === "muted" && "nb-label--muted",
        className,
      )}
    >
      {children}
    </Tag>
  );
}

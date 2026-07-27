import { cn } from "../../lib/utils";

interface NbBlockAmberProps {
  children: React.ReactNode;
  className?: string;
  as?: "div" | "section";
}

export function NbBlockAmber({ children, className, as: Tag = "div" }: NbBlockAmberProps) {
  return <Tag className={cn("nb-block-amber", className)}>{children}</Tag>;
}

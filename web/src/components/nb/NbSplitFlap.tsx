import { cn } from "../../lib/utils";

interface NbSplitFlapProps {
  value: string;
  className?: string;
}

export function NbSplitFlap({ value, className }: NbSplitFlapProps) {
  return (
    <span className={cn("nb-split", className)}>
      <span className="nb-split-inner">{value}</span>
    </span>
  );
}

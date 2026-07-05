import { cn } from "../../lib/utils";

interface NbSplitFlapProps {
  value: string;
  className?: string;
}

export function NbSplitFlap({ value, className }: NbSplitFlapProps) {
  return (
    <span className={cn("nb-splitflap", className)} aria-hidden="true">
      {value.split("").map((ch, i) => (
        <span key={i} className="nb-splitflap-cell">
          <span className="nb-splitflap-top">{ch === "." ? "." : ch === "," ? "," : ch}</span>
          <span className="nb-splitflap-bot">{ch === "." ? "." : ch === "," ? "," : ch}</span>
        </span>
      ))}
    </span>
  );
}

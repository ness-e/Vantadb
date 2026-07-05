import { cn } from "../../lib/utils";

interface NbSectionHeaderProps {
  monoLabel: string;
  headline: string;
  sub?: string;
  align?: "left" | "center";
}

export function NbSectionHeader({
  monoLabel,
  headline,
  sub,
  align = "left",
}: NbSectionHeaderProps) {
  return (
    <div className={cn("nb-section-header", align === "center" && "nb-text-center")}>
      <span className="nb-mono-label">{monoLabel}</span>
      <h2 className="nb-section-headline">{headline}</h2>
      {sub && <p className="nb-section-sub">{sub}</p>}
    </div>
  );
}

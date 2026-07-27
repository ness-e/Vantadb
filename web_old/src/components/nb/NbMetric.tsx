import { cn } from "../../lib/utils";

interface NbMetricProps {
  value: string | number;
  label: string;
  unit?: string;
  className?: string;
}

export function NbMetric({ value, label, unit, className }: NbMetricProps) {
  return (
    <div className={cn("nb-metric", className)}>
      <div className="nb-metric-value">
        {value}
        {unit && <span className="nb-metric-unit">{unit}</span>}
      </div>
      <div className="nb-metric-label">{label}</div>
    </div>
  );
}

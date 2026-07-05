import { cn } from "../../lib/utils";

interface NbTelemetryProps {
  items: string[];
  className?: string;
}

export function NbTelemetry({ items, className }: NbTelemetryProps) {
  return (
    <div className={cn("nb-telemetry", className)}>
      {items.map((item, i) => (
        <span key={i}>{item}</span>
      ))}
    </div>
  );
}

import { memo } from "react";
import { EngineCard } from "./EngineCard";

interface BenchmarkItem {
  value: string;
  label: string;
  unit: string;
}

interface EngineBenchmarkProps {
  panelLabel: string;
  description: string;
  items: BenchmarkItem[];
}

export const EngineBenchmark = memo(function EngineBenchmark({
  panelLabel,
  description,
  items,
}: EngineBenchmarkProps) {
  return (
    <div className="nc-engine-panel nc-engine-section nc-engine-part">
      <div className="nc-engine-panel-label">{panelLabel}</div>
      <p className="nc-engine-slider-desc">{description}</p>
      <div className="nb-grid nb-grid--cols-2">
        {items.map((item) => (
          <EngineCard key={item.label} value={item.value} label={item.label} unit={item.unit} />
        ))}
      </div>
    </div>
  );
});

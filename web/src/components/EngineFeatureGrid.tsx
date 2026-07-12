import type { ReactNode } from "react";

interface EngineFeatureGridProps {
  left: ReactNode;
  right: ReactNode;
  leftLabel: string;
}

export function EngineFeatureGrid({ left, right, leftLabel }: EngineFeatureGridProps) {
  return (
    <div className="nb-grid nb-grid--cols-2 nc-engine-section">
      <div className="nc-engine-panel nc-engine-part nc-engine-radar">
        <div className="nc-engine-panel-label">{leftLabel}</div>
        {left}
      </div>
      <div className="nc-engine-part">
        {right}
      </div>
    </div>
  );
}

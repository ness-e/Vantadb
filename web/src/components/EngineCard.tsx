export interface EngineCardProps {
  value: string;
  label: string;
  unit: string;
}

export function EngineCard({ value, label, unit }: EngineCardProps) {
  return (
    <div className="nc-engine-gauge">
      <div className="nc-engine-gauge-value">{value}</div>
      <span className="nc-engine-gauge-label">{label}</span>
      <span className="nc-engine-gauge-unit">{unit}</span>
    </div>
  );
}

interface NbProgressProps {
  value: number;
  label: string;
  variant?: "amber" | "solid";
}

export function NbProgress({ value, label, variant = "amber" }: NbProgressProps) {
  return (
    <div>
      <div className="flex justify-between mb-2">
        <span className="font-bold font-mono text-micro">{label}</span>
        <span className="font-mono font-black text-micro">{value}%</span>
      </div>
      <div className="nb-progress">
        <div
          className={`nb-progress-bar nb-progress-bar--${variant}`}
          style={{ width: `${Math.min(100, Math.max(0, value))}%` }}
        />
      </div>
    </div>
  );
}

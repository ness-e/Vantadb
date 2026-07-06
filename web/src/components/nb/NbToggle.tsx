import { useState } from "react";

interface NbToggleProps {
  defaultOn?: boolean;
  onChange?: (on: boolean) => void;
  label: string;
}

export function NbToggle({ defaultOn = false, onChange, label }: NbToggleProps) {
  const [on, setOn] = useState(defaultOn);

  const toggle = () => {
    const next = !on;
    setOn(next);
    onChange?.(next);
  };

  return (
    <div className="flex items-center gap-3">
      <button
        type="button"
        className={`nb-toggle${on ? " nb-toggle--on" : ""}`}
        onClick={toggle}
        role="switch"
        aria-checked={on}
        aria-label={label}
      />
      <span className="font-bold font-mono text-micro">{label}</span>
    </div>
  );
}

import { useRef } from "react";
import { useCountUp } from "../../hooks/useCountUp";

interface NbCounterProps {
  target: number;
  suffix?: string;
  label?: string;
}

export function NbCounter({ target, suffix = "", label }: NbCounterProps) {
  const ref = useRef<HTMLDivElement>(null);
  const value = useCountUp(target, ref, { duration: 1.5 });

  return (
    <div ref={ref}>
      <div className="nb-counter">{Math.round(value).toLocaleString()}{suffix}</div>
      {label && <div className="nb-counter-label">{label}</div>}
    </div>
  );
}

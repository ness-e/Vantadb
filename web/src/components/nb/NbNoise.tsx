import { cn } from "../../lib/utils";

interface NbNoiseProps {
  className?: string;
}

export function NbNoise({ className }: NbNoiseProps) {
  return <div className={cn("nb-noise", className)} aria-hidden="true" />;
}

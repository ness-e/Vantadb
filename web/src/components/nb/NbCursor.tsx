import { cn } from "../../lib/utils";

interface NbCursorProps {
  className?: string;
}

export function NbCursor({ className }: NbCursorProps) {
  return <span className={cn("nb-cursor", className)} aria-hidden="true" />;
}

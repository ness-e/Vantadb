import { cn } from "../../lib/utils";

interface NbLogLineProps {
  message: string;
  level?: "info" | "warn" | "ok" | "error" | "debug";
  className?: string;
}

const LEVEL_MAP = {
  info: "INFO",
  warn: "WARN",
  ok: "OK",
  error: "ERR",
  debug: "DEBUG",
} as const;

export function NbLogLine({ message, level = "info", className }: NbLogLineProps) {
  return (
    <div
      className={cn(
        "nb-log-line",
        level === "info" && "nb-log-line--info",
        level === "warn" && "nb-log-line--warn",
        level === "ok" && "nb-log-line--ok",
        level === "error" && "nb-log-line--error",
        className,
      )}
      data-level={LEVEL_MAP[level]}
    >
      {message}
    </div>
  );
}

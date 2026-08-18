// Chips de encoding redundante por op/outcome (VS-15, D/a11y): color + ícono +
// texto, para que ningún estado dependa solo del color (VS-18 transversal usa
// el mismo patrón). Reutilizados por Timeline y la tabla de ActivityPanel.
import { opMeta, outcomeMeta, type OpTone } from "./logic";

/** Clases por tone — la distinción vive en el border/background/text, no solo
 * en el color; el label y el ícono acompañan siempre. */
const TONE_CLASS: Record<OpTone, string> = {
  neutral: "border-2 border-foreground bg-background text-foreground",
  batch: "border-2 border-foreground bg-neon text-background",
  danger: "border-2 border-foreground bg-foreground text-background",
  transfer: "border-2 border-foreground bg-muted text-foreground",
  unknown: "border-2 border-foreground bg-background text-muted-foreground",
};

export function OpChip({ op }: { op: string }) {
  const meta = opMeta(op);
  return (
    <span
      className={`inline-flex shrink-0 items-center gap-1 px-1.5 py-0.5 font-tech text-[10px] font-bold uppercase tracking-wider ${TONE_CLASS[meta.tone]}`}
      title={`operación: ${op}`}
    >
      <span aria-hidden="true">{meta.icon}</span>
      {meta.label}
    </span>
  );
}

export function OutcomeBadge({ outcome }: { outcome: string }) {
  const meta = outcomeMeta(outcome);
  return (
    <span
      className={`inline-flex shrink-0 items-center gap-1 font-tech text-[10px] font-bold uppercase tracking-wider ${
        meta.err ? "text-neon" : "text-muted-foreground"
      }`}
      title={`outcome: ${outcome}`}
    >
      <span aria-hidden="true">{meta.icon}</span>
      {meta.label}
    </span>
  );
}
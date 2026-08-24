// Timeline (VS-15): el flujo de auditoría unificado (todas las ops de todos los
// namespaces — escrituras, borrados, export/import) agrupado por hora o por día.
// Recibe los mismos eventos ya cargados por ActivityPanel (sin fetch extra);
// el filtrado por namespace/op/outcome ocurre aguas arriba en Rust (VS-12).
import type { AuditEvent } from "../../vanta";
import { eventClock, groupByBucket, opFamily, type BucketGranularity, type OpFamily } from "./logic";
import { OpChip, OutcomeBadge } from "./EventChip";

interface Props {
  events: AuditEvent[];
  granularity: BucketGranularity;
  onInspect: (e: AuditEvent) => void;
  onPeek: (e: AuditEvent | null) => void;
}

export default function Timeline({ events, granularity, onInspect, onPeek }: Props) {
  const buckets = groupByBucket(events, granularity);

  if (buckets.length === 0) return null;

  return (
    <ol className="space-y-5" aria-label="Cronología de actividad">
      {buckets.map((bucket) => (
        <li key={bucket.key}>
          <div className="flex flex-wrap items-baseline gap-2 border-b-2 border-foreground pb-1">
            <h3 className="font-display text-2xl text-stencil">{bucket.label}</h3>
            <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              {bucket.events.length} evento{bucket.events.length === 1 ? "" : "s"} ·{" "}
              {familySummary(bucket.events)}
            </span>
          </div>
          <ul className="mt-1 divide-y divide-foreground">
            {bucket.events.map((e, i) => (
              <li key={`${e.timestamp}-${i}`}>
                <button
                  type="button"
                  onClick={() => onInspect(e)}
                  onMouseEnter={() => onPeek(e)}
                  onMouseLeave={() => onPeek(null)}
                  className="flex w-full items-center gap-2 px-1 py-1.5 text-left hover:bg-muted"
                  title={
                    e.namespace === "N/A" || e.key === "N/A"
                      ? `${e.op} — operación sin registro asociado`
                      : `clic para abrir ${e.namespace}:${e.key} en Inspector`
                  }
                >
                  <OpChip op={e.op} />
                  <code className="min-w-0 flex-1 truncate font-tech text-[12px]">{e.key}</code>
                  <span className="shrink-0 border-2 border-foreground bg-background px-1.5 py-0.5 font-tech text-[10px]">
                    {e.namespace}
                  </span>
                  <OutcomeBadge outcome={e.outcome} />
                  <span className="shrink-0 font-tech text-[10px] text-muted-foreground">
                    {eventClock(e.timestamp)}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        </li>
      ))}
    </ol>
  );
}

/** Resumen por familia (escrituras/borrados/export-import) para el header del
 * bucket — alinea el Timeline con el contrato (b) sin duplicar el chip por fila. */
const FAMILY_LABEL: Record<OpFamily, string> = {
  write: "escrituras",
  delete: "borrados",
  transfer: "export/import",
};

function familySummary(events: AuditEvent[]): string {
  const counts: Record<OpFamily, number> = { write: 0, delete: 0, transfer: 0 };
  for (const e of events) counts[opFamily(e.op)] += 1;
  return (Object.keys(FAMILY_LABEL) as OpFamily[])
    .filter((f) => counts[f] > 0)
    .map((f) => `${counts[f]} ${FAMILY_LABEL[f]}`)
    .join(" · ");
}
// Derived KPI cards (ADMIN-05). Consumes the operational metrics snapshot via
// the shared poll hook (useMetricsPoll, DESKTOP-29) and renders value + label
// + CSS bar sparkline. The shared history window doubles as the sparkline series.
import { OperationalMetrics } from "../vanta";
import { useMetricsPoll } from "../hooks/useMetricsPoll";

/** Guarded ratio — avoids NaN/Infinity on empty counters. */
function ratio(num: number, den: number): number {
  return den > 0 ? num / den : 0;
}

function pct(v: number): string {
  return `${(v * 100).toFixed(1)}%`;
}

function fmtBytes(v: number): string {
  return v >= 1 << 20 ? `${(v / (1 << 20)).toFixed(1)} MiB` : `${(v / 1024).toFixed(1)} KiB`;
}

interface KpiDatum {
  label: string;
  current: string;
  series: number[];
}

function computeKpis(history: OperationalMetrics[]): KpiDatum[] {
  const last = history[history.length - 1];

  const memEff = (m: OperationalMetrics) => ratio(m.mmap_resident_bytes ?? 0, m.process_rss_bytes);
  const hybridShare = (m: OperationalMetrics) =>
    ratio(m.planner_hybrid_queries, m.planner_hybrid_queries + m.planner_text_only_queries);
  const importErr = (m: OperationalMetrics) => ratio(m.import_errors, m.records_imported);
  const walEff = (m: OperationalMetrics) => ratio(m.wal_records_replayed, m.wal_replay_ms);
  const hnswPerNode = (m: OperationalMetrics) => ratio(m.hnsw_logical_bytes, m.hnsw_nodes_count);

  return [
    { label: "Eficiencia de memoria", current: pct(memEff(last)), series: history.map(memEff) },
    { label: "Consultas híbridas", current: pct(hybridShare(last)), series: history.map(hybridShare) },
    { label: "Errores de importación", current: pct(importErr(last)), series: history.map(importErr) },
    { label: "Eficiencia WAL", current: `${walEff(last).toFixed(2)} rec/ms`, series: history.map(walEff) },
    { label: "Bytes HNSW / nodo", current: fmtBytes(hnswPerNode(last)), series: history.map(hnswPerNode) },
  ];
}

function Sparkline({ values }: { values: number[] }) {
  const max = Math.max(0, ...values);
  return (
    <div
      className="mt-auto flex h-7 items-end gap-0.5 border-b-2 border-foreground pb-0.5"
      role="img"
      aria-label={`tendencia: ${values.map((v) => v.toFixed(3)).join(", ")}`}
    >
      {values.map((v, i) => {
        // Min 4px stub so zero values stay visible; scale to window max for shape.
        const h = max > 0 ? Math.max(4, Math.round((v / max) * 24)) : 4;
        return <span key={i} className="min-w-0.5 flex-1 border border-foreground bg-neon" style={{ height: `${h}px` }} />;
      })}
    </div>
  );
}

export default function KpiCards() {
  const { history, error } = useMetricsPoll();

  if (history.length === 0) {
    return (
      <section
        aria-label="KPIs"
        className="border-[3px] border-foreground bg-card p-4 shadow-ink"
      >
        <h2 className="m-0 font-tech text-xs uppercase tracking-widest">KPIs</h2>
        {/* Error amigable: el detalle crudo queda en consola (useMetricsPoll). */}
        <p className="text-muted-foreground">
          {error ? "sin métricas todavía — reintentando…" : "Esperando métricas…"}
        </p>
      </section>
    );
  }

  return (
    <section       aria-label="KPIs derivados" className="grid grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-3">
      {computeKpis(history).map((k) => (
        <article
          key={k.label}
          className="flex flex-col gap-1.5 border-[3px] border-foreground bg-card p-3 shadow-ink"
        >
          <div className="text-2xl font-bold leading-none tracking-tight">{k.current}</div>
          <div className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">{k.label}</div>
          <Sparkline values={k.series} />
        </article>
      ))}
    </section>
  );
}

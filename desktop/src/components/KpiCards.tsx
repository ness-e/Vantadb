// Derived KPI cards (ADMIN-05). Consumes the operational metrics snapshot via
// the typed bridge (vanta.ts) and renders value + label + CSS bar sparkline.
// Self-contained 5s poll with a short ring buffer so sparklines show a trend;
// ADMIN-04 owns the full dashboard polling when it lands.
import { useEffect, useState } from "react";
import { metrics, OperationalMetrics, vantaErrorMessage } from "../vanta";

const POLL_MS = 5000;
const WINDOW = 12;

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
    { label: "Memory efficiency", current: pct(memEff(last)), series: history.map(memEff) },
    { label: "Hybrid query share", current: pct(hybridShare(last)), series: history.map(hybridShare) },
    { label: "Import error rate", current: pct(importErr(last)), series: history.map(importErr) },
    { label: "WAL efficiency", current: `${walEff(last).toFixed(2)} rec/ms`, series: history.map(walEff) },
    { label: "HNSW bytes / node", current: fmtBytes(hnswPerNode(last)), series: history.map(hnswPerNode) },
  ];
}

function Sparkline({ values }: { values: number[] }) {
  const max = Math.max(0, ...values);
  return (
    <div
      className="sparkline"
      role="img"
      aria-label={`trend: ${values.map((v) => v.toFixed(3)).join(", ")}`}
    >
      {values.map((v, i) => {
        // Min 4px stub so zero values stay visible; scale to window max for shape.
        const h = max > 0 ? Math.max(4, Math.round((v / max) * 24)) : 4;
        return <span key={i} className="spark-bar" style={{ height: `${h}px` }} />;
      })}
    </div>
  );
}

export default function KpiCards() {
  const [history, setHistory] = useState<OperationalMetrics[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    let timer: number | undefined;

    async function poll() {
      try {
        const m = await metrics();
        if (!alive) return;
        setHistory((h) => [...h.slice(-(WINDOW - 1)), m]);
        setError(null);
      } catch (e) {
        if (alive) setError(vantaErrorMessage(e));
      }
    }

    poll();
    timer = window.setInterval(poll, POLL_MS);
    return () => {
      alive = false;
      if (timer) window.clearInterval(timer);
    };
  }, []);

  if (history.length === 0) {
    return (
      <section className="panel" aria-label="KPIs">
        <h2>KPIs</h2>
        <p className="muted">{error ? error : "Waiting for metrics…"}</p>
      </section>
    );
  }

  return (
    <section className="kpi-grid" aria-label="Derived KPIs">
      {computeKpis(history).map((k) => (
        <article key={k.label} className="kpi-card">
          <div className="kpi-value">{k.current}</div>
          <div className="kpi-label">{k.label}</div>
          <Sparkline values={k.series} />
        </article>
      ))}
    </section>
  );
}

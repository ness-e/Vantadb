// Metro-style operational metrics grid (ADMIN-04).
// Polls `vanta_metrics` (shared bridge) every POLL_MS and renders one tile per
// key metric: value + delta vs the previous snapshot + a 3-point trend arrow.
// ADMIN-02 is not committed yet, so the poll/delta lives inline here; when
// useMetrics lands as a commit this component can be slimmed to consume it.
import { useEffect, useRef, useState } from "react";
import { HealthReport, metrics, OperationalMetrics, vantaErrorMessage } from "../vanta";

const POLL_MS = 4000;

function fmtBytes(n: number): string {
  if (n >= 1e9) return `${(n / 1e9).toFixed(2)} GB`;
  if (n >= 1e6) return `${(n / 1e6).toFixed(1)} MB`;
  if (n >= 1e3) return `${(n / 1e3).toFixed(0)} KB`;
  return `${n} B`;
}

function fmtCount(n: number): string {
  if (n >= 1e9) return `${(n / 1e9).toFixed(2)}B`;
  if (n >= 1e6) return `${(n / 1e6).toFixed(1)}M`;
  if (n >= 1e3) return `${(n / 1e3).toFixed(1)}K`;
  return `${n}`;
}

function totalQueries(m: OperationalMetrics): number {
  return m.planner_hybrid_queries + m.planner_text_only_queries + m.planner_vector_only_queries;
}

interface Tile {
  key: string;
  title: string;
  value: string;
  /** Signed delta since last poll, e.g. "+1.2K". */
  delta: string;
  trend: "up" | "down" | "flat";
  muted?: string;
}

interface Props {
  health: HealthReport | null;
  healthStatus: "ok" | "warn" | "err" | "idle";
  activeName: string | null;
}

export default function MetricsGrid({ health, healthStatus, activeName }: Props) {
  // Last snapshots, newest last. Trend = delta(now,prev) vs delta(prev,prevprev).
  const history = useRef<OperationalMetrics[]>([]);
  const [latest, setLatest] = useState<OperationalMetrics | null>(null);
  const [polledAt, setPolledAt] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const tick = async () => {
      try {
        const m = await metrics();
        if (cancelled) return;
        history.current = [...history.current.slice(-2), m];
        setLatest(m);
        setPolledAt(Date.now());
        setError(null);
      } catch (e) {
        if (!cancelled) setError(vantaErrorMessage(e));
      }
    };
    tick();
    const id = setInterval(tick, POLL_MS);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, []);

  function deltaAndTrend(get: (m: OperationalMetrics) => number, fmt: (n: number) => string): Pick<Tile, "delta" | "trend"> {
    const h = history.current;
    if (h.length < 2) return { delta: "—", trend: "flat" };
    const now = get(h[h.length - 1]);
    const prev = get(h[h.length - 2]);
    const d = now - prev;
    if (d === 0) return { delta: "0", trend: "flat" };
    const delta = `${d > 0 ? "+" : "−"}${fmt(Math.abs(d))}`;
    if (h.length < 3) return { delta, trend: d > 0 ? "up" : "down" };
    const prevDelta = prev - get(h[h.length - 3]);
    return { delta, trend: d > prevDelta ? "up" : d < prevDelta ? "down" : "flat" };
  }

  const tiles: Tile[] = latest
    ? [
        {
          key: "rss",
          title: "RSS",
          value: fmtBytes(latest.process_rss_bytes),
          ...deltaAndTrend((m) => m.process_rss_bytes, fmtBytes),
          muted: "resident set size",
        },
        {
          key: "records",
          title: "Records",
          value: fmtCount(latest.records_imported),
          ...deltaAndTrend((m) => m.records_imported, fmtCount),
          muted: latest.import_errors > 0 ? `${latest.import_errors} import errors` : "records imported",
        },
        {
          key: "queries",
          title: "Queries",
          value: fmtCount(totalQueries(latest)),
          ...deltaAndTrend(totalQueries, fmtCount),
          muted: `${fmtCount(latest.text_lexical_queries)} lexical`,
        },
        {
          key: "scans",
          title: "Scans",
          value: fmtCount(latest.derived_prefix_scans),
          ...deltaAndTrend((m) => m.derived_prefix_scans, fmtCount),
          muted: `${latest.derived_full_scan_fallbacks} full-scan fallbacks`,
        },
        {
          key: "wal",
          title: "WAL Replay",
          value: fmtCount(latest.wal_records_replayed),
          ...deltaAndTrend((m) => m.wal_records_replayed, fmtCount),
          muted: `replay ${latest.wal_replay_ms}ms`,
        },
        {
          key: "text",
          title: "Text Index",
          value: fmtCount(latest.text_postings_written),
          ...deltaAndTrend((m) => m.text_postings_written, fmtCount),
          muted: `${latest.text_index_repairs} repairs`,
        },
      ]
    : [];

  const trendGlyph = { up: "▲", down: "▼", flat: "—" } as const;

  return (
    <section className="panel metrics" aria-label="Operational metrics">
      <div className="panel-head">
        <h2>Metrics</h2>
        <div className="row metrics-head-right">
          <span className="muted polled">
            {activeName ? activeName : "no backend"}
            {polledAt ? ` · ${new Date(polledAt).toLocaleTimeString()}` : " · waiting…"}
          </span>
          <span className="health-badge" data-status={healthStatus} title="vanta_health">
            {healthStatus === "idle"
              ? "—"
              : health
                ? `${health.backend} · ${health.latency_ms}ms`
                : "check"}
          </span>
        </div>
      </div>

      {error && <p className="muted metrics-error">metrics unavailable: {error}</p>}

      {tiles.length === 0 ? (
        <p className="muted metrics-error">Waiting for first metrics snapshot…</p>
      ) : (
        <div className="metrics-grid">
          {tiles.map((t) => (
            <div key={t.key} className="tile">
              <h3>{t.title}</h3>
              <p className="tile-value">{t.value}</p>
              <p className="tile-delta">
                <span className={`trend-${t.trend}`} aria-hidden="true">
                  {trendGlyph[t.trend]}
                </span>
                <span>{t.delta}</span>
                {t.muted && <span className="muted">· {t.muted}</span>}
              </p>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

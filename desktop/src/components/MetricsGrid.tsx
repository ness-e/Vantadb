// Metro-style operational metrics grid (ADMIN-04).
// Renders one tile per key metric: value + delta vs the previous snapshot +
// a 3-point trend arrow. Polling is shared via useMetricsPoll (DESKTOP-29).
import { HealthReport, OperationalMetrics } from "../vanta";
import { useMetricsPoll } from "../hooks/useMetricsPoll";
// UX-15: fmtBytes compartido (antes había un local idéntico acá).
import { fmtBytes } from "../lib/format";

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
  // Shared poll history, newest last. Trend = delta(now,prev) vs delta(prev,prevprev).
  const { history: h, error, polledAt } = useMetricsPoll();
  const latest = h[h.length - 1] ?? null;

  function deltaAndTrend(get: (m: OperationalMetrics) => number, fmt: (n: number) => string): Pick<Tile, "delta" | "trend"> {
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
          muted: "memoria residente",
        },
        {
          key: "records",
          title: "Registros",
          value: fmtCount(latest.records_imported),
          ...deltaAndTrend((m) => m.records_imported, fmtCount),
          muted: latest.import_errors > 0 ? `${latest.import_errors} errores de importación` : "registros importados",
        },
        {
          key: "queries",
          title: "Consultas",
          value: fmtCount(totalQueries(latest)),
          ...deltaAndTrend(totalQueries, fmtCount),
          muted: `${fmtCount(latest.text_lexical_queries)} léxicas`,
        },
        {
          key: "scans",
          title: "Escaneos",
          value: fmtCount(latest.derived_prefix_scans),
          ...deltaAndTrend((m) => m.derived_prefix_scans, fmtCount),
          muted: `${latest.derived_full_scan_fallbacks} fallbacks de escaneo completo`,
        },
        {
          key: "wal",
          title: "Replay WAL",
          value: fmtCount(latest.wal_records_replayed),
          ...deltaAndTrend((m) => m.wal_records_replayed, fmtCount),
          muted: `replay ${latest.wal_replay_ms}ms`,
        },
        {
          key: "text",
          title: "Índice de texto",
          value: fmtCount(latest.text_postings_written),
          ...deltaAndTrend((m) => m.text_postings_written, fmtCount),
          muted: `${latest.text_index_repairs} reparaciones`,
        },
      ]
    : [];

  const trendGlyph = { up: "▲", down: "▼", flat: "—" } as const;
  const trendClass = { up: "text-neon", down: "text-foreground", flat: "text-muted-foreground" } as const;

  return (
    <section
      aria-label="Métricas operativas"
      className="border-[3px] border-foreground bg-card p-4 shadow-ink"
    >
      <div className="mb-3 flex items-center justify-between gap-2">
        <h2 className="m-0 font-tech text-xs uppercase tracking-widest">Métricas</h2>
        <div className="flex items-center gap-2">
          <span className="text-muted-foreground text-sm">
            {activeName ? activeName : "no backend"}
            {polledAt ? ` · ${new Date(polledAt).toLocaleTimeString()}` : " · waiting…"}
          </span>
          <span
            className={`border-2 px-2.5 py-1 text-xs ${
              healthStatus === "idle"
                ? "text-muted-foreground"
                : healthStatus === "ok"
                  ? "bg-paper text-foreground"
                  : // UX-15: warn y err compartían el mismo look ámbar — err ahora
                    // usa el token destructivo (antes era igual a warn).
                    healthStatus === "err"
                    ? "border-destructive bg-destructive/10 text-destructive"
                    : "border-neon bg-neon/10 text-accent-text"
            }`}
            data-status={healthStatus}
            title="vanta_health"
          >
            {healthStatus === "idle"
              ? "—"
              : health
                ? `${health.backend} · ${health.latency_ms}ms`
                : "check"}
          </span>
        </div>
      </div>

      {/* Error amigable: el detalle técnico (TypeError, HTTP 4xx/5xx crudo)
          queda en consola, no en la cara del usuario (visual-critique). */}
      {error && (
        <p className="mt-2 text-sm text-muted-foreground">
          métricas no disponibles todavía — reintentando automáticamente
        </p>
      )}

      {tiles.length === 0 ? (
        <p className="mt-2 text-sm text-muted-foreground">Esperando el primer snapshot de métricas…</p>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {tiles.map((t) => (
            <div key={t.key} className="press border-[3px] border-foreground bg-card px-4 py-3.5">
              <h3 className="mb-2 mt-0 font-tech text-xs uppercase tracking-widest text-muted-foreground">
                {t.title}
              </h3>
              <p className="m-0 text-[1.6rem] font-bold leading-tight tracking-tight">{t.value}</p>
              <p className="mb-0 mt-2 flex items-center gap-1.5 text-sm">
                <span className={trendClass[t.trend]} aria-hidden="true">
                  {trendGlyph[t.trend]}
                </span>
                <span>{t.delta}</span>
                {t.muted && <span className="text-muted-foreground">· {t.muted}</span>}
              </p>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

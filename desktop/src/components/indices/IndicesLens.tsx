// IndicesLens (FEAT-02): real ÍNDICES surface replacing the VS-03 placeholder.
// Paints what the core actually exposes via the shared bridge (vanta.ts) on all
// three transports (Tauri native / HTTP REST / WASM):
//   · Salud — health report (prop from the shell state, same source as topbar)
//   · Namespaces — per-namespace counts + expiry buckets (namespaceStats, with
//     WASM fallback to list() counts, same pattern as WorkspaceShell)
//   · Índice vectorial (HNSW) — hnsw_nodes_count / hnsw_logical_bytes / ann_rebuild
//   · Índice de texto (BM25) — postings / queries / candidates / repairs
//   · WAL — startup replay counters (no live status: documented gap)
// Charts are simple horizontal bars reusing the ScoreBars visual language
// (length primary, color secondary; border-2 + foreground fill).
//
// "No mentir en UI": every tile reads a real VantaOperationalMetrics /
// VantaNamespaceStats field. dims + live LSM/WAL status are NOT exposed by the
// core → rendered as "—" and listed in CORE_GAPS (never invented numbers).
import { useEffect, useState } from "react";
import {
  HealthReport,
  list,
  namespaceStats,
  type NamespaceStatsMap,
} from "../../vanta";
import { useMetricsPoll } from "../../hooks/useMetricsPoll";
import {
  CORE_GAPS,
  namespaceBars,
  namespaceBarsFromCounts,
  textIndexTiles,
  vectorIndexTiles,
  walTiles,
  type IndexTile,
  type NamespaceBar,
} from "./indices-core";

const POLL_MS = 4000;

interface Props {
  health: HealthReport | null;
  healthStatus: "ok" | "warn" | "err" | "idle";
  activeName: string | null;
}

function Tile({ t }: { t: IndexTile }) {
  return (
    <div className="border-2 border-foreground bg-card p-3" title={t.gap ? CORE_GAPS[0].detail : undefined}>
      <div className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">{t.label}</div>
      <div className={`font-display text-2xl leading-none ${t.gap ? "text-muted-foreground" : "text-foreground"}`}>
        {t.value}
      </div>
      {t.muted && <div className="mt-1 font-tech text-[9px] text-muted-foreground">{t.muted}</div>}
    </div>
  );
}

export default function IndicesLens({ health, healthStatus, activeName }: Props) {
  // Shared vanta_metrics poll (DESKTOP-29) — no local metrics interval.
  const { history, error } = useMetricsPoll();
  const snapshot = history[history.length - 1] ?? null;
  const [bars, setBars] = useState<NamespaceBar[]>([]);

  // Per-namespace stats: real endpoint on native/HTTP; WASM falls back to
  // list() counts (expiry buckets = null) — same pattern as WorkspaceShell.
  useEffect(() => {
    let alive = true;
    const load = async () => {
      try {
        const stats: NamespaceStatsMap = await namespaceStats();
        if (alive) setBars(namespaceBars(stats));
      } catch {
        if (!alive) return;
        try {
          const records = await list({ limit: 500 });
          if (!alive) return;
          const counts: Record<string, number> = {};
          for (const r of records) counts[r.namespace] = (counts[r.namespace] ?? 0) + 1;
          setBars(namespaceBarsFromCounts(counts));
        } catch {
          if (alive) setBars([]);
        }
      }
    };
    load();
    const id = setInterval(load, POLL_MS);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, []);

  const ok = healthStatus === "ok";

  return (
    <div className="space-y-5">
      <div className="flex items-center justify-between">
        <h2 className="font-display text-2xl text-stencil">ÍNDICES</h2>
        <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
          {activeName ?? "sin backend"} · poll 4s
        </span>
      </div>

      {error && (
        <p role="alert" className="border-2 border-foreground bg-card px-3 py-2 font-tech text-[11px]">
          metrics unavailable: {error}
        </p>
      )}

      {/* Salud */}
      <section aria-label="Salud" className="border-2 border-foreground bg-card p-4">
        <div className="font-tech text-[10px] uppercase tracking-widest text-neon">Salud</div>
        <div className="mt-2 flex flex-wrap items-center gap-4">
          <span
            className={`border-2 border-foreground px-2 py-1 font-tech text-[11px] uppercase ${
              ok ? "bg-neon text-background" : "bg-background text-muted-foreground"
            }`}
          >
            {ok ? "● healthy" : healthStatus === "idle" ? "○ idle" : "○ offline"}
          </span>
          {health && (
            <span className="font-tech text-[11px] text-muted-foreground">
              {health.backend} · {health.latency_ms}ms · checked {new Date(health.checked_at_ms).toLocaleTimeString()}
            </span>
          )}
          {snapshot && (
            <span className="font-tech text-[11px] text-muted-foreground">startup {snapshot.startup_ms}ms</span>
          )}
        </div>
      </section>

      {/* Namespaces */}
      <section aria-label="Namespaces" className="border-2 border-foreground bg-card p-4">
        <div className="font-tech text-[10px] uppercase tracking-widest text-neon">
          Namespaces {bars.length > 0 ? `(${bars.length})` : ""}
        </div>
        {bars.length === 0 ? (
          <p className="mt-2 font-tech text-[11px] text-muted-foreground">sin registros</p>
        ) : (
          <ul className="mt-3 space-y-3">
            {bars.map((b) => (
              <li key={b.name}>
                <div className="flex items-baseline justify-between gap-2">
                  <span className="truncate text-sm font-semibold">{b.name}</span>
                  <span className="shrink-0 font-tech text-[10px] text-muted-foreground">
                    {b.expiringSoon != null && b.expired != null && (b.expiringSoon > 0 || b.expired > 0)
                      ? `${b.expiringSoon} expiran · ${b.expired} expirados`
                      : b.expiringSoon != null
                        ? "sin expiración"
                        : "sin stats"}
                  </span>
                </div>
                {/* Barra estilo ScoreBars: longitud primaria, color secundario. */}
                <div className="mt-1 flex items-center gap-2">
                  <div
                    role="img"
                    aria-label={`${b.name}: ${b.count} registros (${b.widthPct.toFixed(0)}% del máximo)`}
                    className="relative h-4 min-w-0 flex-1 overflow-hidden border-2 border-foreground bg-background"
                  >
                    <div
                      className="absolute inset-y-0 left-0 bg-foreground"
                      style={{ width: `${b.widthPct}%` }}
                    />
                  </div>
                  <span className="w-12 shrink-0 text-right font-display text-base leading-none">{b.count}</span>
                </div>
              </li>
            ))}
          </ul>
        )}
      </section>

      {/* Índices */}
      {snapshot && (
        <>
          <section aria-label="Índice vectorial" className="border-2 border-foreground bg-card p-4">
            <div className="font-tech text-[10px] uppercase tracking-widest text-neon">Índice vectorial · HNSW</div>
            <div className="mt-3 grid grid-cols-2 gap-3 md:grid-cols-4">
              {vectorIndexTiles(snapshot).map((t) => (
                <Tile key={t.key} t={t} />
              ))}
            </div>
          </section>

          <section aria-label="Índice de texto" className="border-2 border-foreground bg-card p-4">
            <div className="font-tech text-[10px] uppercase tracking-widest text-neon">Índice de texto · BM25</div>
            <div className="mt-3 grid grid-cols-2 gap-3 md:grid-cols-4">
              {textIndexTiles(snapshot).map((t) => (
                <Tile key={t.key} t={t} />
              ))}
            </div>
          </section>

          <section aria-label="WAL" className="border-2 border-foreground bg-card p-4">
            <div className="font-tech text-[10px] uppercase tracking-widest text-neon">WAL</div>
            <div className="mt-3 grid grid-cols-2 gap-3 md:grid-cols-4">
              {walTiles(snapshot).map((t) => (
                <Tile key={t.key} t={t} />
              ))}
            </div>
          </section>
        </>
      )}

      {/* Gaps documentados — nunca inventar métricas en la UI. */}
      <section aria-label="Métricas no expuestas por el core" className="border-2 border-dashed border-muted-foreground p-4">
        <div className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
          no expuesto por el core · follow-up
        </div>
        <ul className="mt-2 space-y-1">
          {CORE_GAPS.map((g) => (
            <li key={g.label} className="font-tech text-[11px] text-muted-foreground">
              <span className="text-foreground">{g.label}</span> — {g.detail}
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
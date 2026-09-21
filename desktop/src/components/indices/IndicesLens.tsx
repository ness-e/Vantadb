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
import LensShell from "../layout/LensShell";
import { tp, tt, type DesktopLang } from "../../i18n";
import { connectionPrefs } from "../../store/connections";
import {
  CORE_GAPS,
  coreGaps,
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
  lang?: DesktopLang;
}

function Tile({ t, lang }: { t: IndexTile; lang: DesktopLang }) {
  // Title del gap: detalle en el idioma activo (CORE_GAPS es el default ES).
  const gaps = lang === "es" ? CORE_GAPS : coreGaps(lang);
  return (
    <div className="border-2 border-foreground bg-card p-3" title={t.gap ? gaps[0].detail : undefined}>
      <div className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">{t.label}</div>
      <div className={`font-display text-2xl leading-none ${t.gap ? "text-muted-foreground" : "text-foreground"}`}>
        {t.value}
      </div>
      {t.muted && <div className="mt-1 font-tech text-[9px] text-muted-foreground">{t.muted}</div>}
    </div>
  );
}

export default function IndicesLens({ health, healthStatus, activeName, lang = connectionPrefs.get().lang ?? "es" }: Props) {
  // Shared vanta_metrics poll (DESKTOP-29) — no local metrics interval.
  const { history, error } = useMetricsPoll();
  const snapshot = history[history.length - 1] ?? null;
  const [bars, setBars] = useState<NamespaceBar[]>([]);
  // UX-15: skeleton en la primera carga — antes `bars=[]` mostraba "sin
  // registros" mientras el primer poll corría (empty state mentiroso).
  const [loading, setLoading] = useState(true);

  // Per-namespace stats: real endpoint on native/HTTP; WASM falls back to
  // list() counts (expiry buckets = null) — same pattern as WorkspaceShell.
  useEffect(() => {
    let alive = true;
    const load = async () => {
      setLoading(true);
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
      } finally {
        if (alive) setLoading(false);
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
      {/* Header (UX-01: LensShell compartido) */}
      <LensShell title="ÍNDICES" meta={tp(lang, "indices.meta", "{b} · poll 4s", { b: activeName ?? tt(lang, "indices.noBackend", "sin backend") })} />

      {error && (
        <p role="alert" className="border-2 border-foreground bg-card px-3 py-2 font-tech text-[11px]">
          {tp(lang, "indices.metricsErr", "métricas no disponibles: {e}", { e: error })}
        </p>
      )}

      {/* Salud */}
      <section aria-label={tt(lang, "indices.healthAria", "Salud")} className="border-2 border-foreground bg-card p-4">
        <div className="font-tech text-[10px] uppercase tracking-widest text-neon">{tt(lang, "indices.healthTitle", "Salud")}</div>
        <div className="mt-2 flex flex-wrap items-center gap-4">
          <span
            className={`border-2 border-foreground px-2 py-1 font-tech text-[11px] uppercase ${
              ok ? "bg-neon text-background" : "bg-background text-muted-foreground"
            }`}
          >
            {ok ? tt(lang, "indices.healthy", "● healthy") : healthStatus === "idle" ? tt(lang, "indices.idle", "○ idle") : tt(lang, "indices.offline", "○ offline")}
          </span>
          {health && (
            <span className="font-tech text-[11px] text-muted-foreground">
              {health.backend} · {health.latency_ms}ms · checked {new Date(health.checked_at_ms).toLocaleTimeString()}
            </span>
          )}
          {snapshot && (
            <span className="font-tech text-[11px] text-muted-foreground">{tp(lang, "indices.startup", "arranque {ms}ms", { ms: String(snapshot.startup_ms) })}</span>
          )}
        </div>
      </section>

      {/* Namespaces */}
      <section aria-label={tt(lang, "indices.nsAria", "Namespaces")} className="border-2 border-foreground bg-card p-4">
        <div className="font-tech text-[10px] uppercase tracking-widest text-neon">
          {tt(lang, "indices.nsTitle", "Namespaces")} {bars.length > 0 ? `(${bars.length})` : ""}
        </div>
        {/* UX-15: skeleton mientras el primer poll no llega (no mentir "sin
          registros" durante la carga). */}
        {loading ? (
          <div className="mt-3 space-y-3" role="status" aria-label={tt(lang, "indices.loadingNsAria", "Cargando namespaces")}>
            {[0, 1, 2].map((i) => (
              <div key={i} className="h-9 animate-pulse border-2 border-foreground bg-muted" />
            ))}
          </div>
        ) : bars.length === 0 ? (
          <p className="mt-2 font-tech text-[11px] text-muted-foreground">{tt(lang, "indices.noRecords", "sin registros")}</p>
        ) : (
          <ul className="mt-3 space-y-3">
            {bars.map((b) => (
              <li key={b.name}>
                <div className="flex items-baseline justify-between gap-2">
                  <span className="truncate text-sm font-semibold">{b.name}</span>
                  <span className="shrink-0 font-tech text-[10px] text-muted-foreground">
                    {b.expiringSoon != null && b.expired != null && (b.expiringSoon > 0 || b.expired > 0)
                      ? tp(lang, "indices.expiring", "{s} expiran · {e} expirados", { s: String(b.expiringSoon), e: String(b.expired) })
                      : b.expiringSoon != null
                        ? tt(lang, "indices.noExpiry", "sin expiración")
                        : tt(lang, "indices.noStats", "sin stats")}
                  </span>
                </div>
                {/* Barra estilo ScoreBars: longitud primaria, color secundario. */}
                <div className="mt-1 flex items-center gap-2">
                  <div
                    role="img"
                    aria-label={tp(lang, "indices.barAria", "{n}: {c} registros ({p}% del máximo)", { n: b.name, c: String(b.count), p: b.widthPct.toFixed(0) })}
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
          <section aria-label={tt(lang, "indices.vecAria", "Índice vectorial")} className="border-2 border-foreground bg-card p-4">
            <div className="font-tech text-[10px] uppercase tracking-widest text-neon">{tt(lang, "indices.vecTitle", "Índice vectorial · HNSW")}</div>
            <div className="mt-3 grid grid-cols-2 gap-3 md:grid-cols-4">
              {vectorIndexTiles(snapshot, lang).map((t) => (
                <Tile key={t.key} t={t} lang={lang} />
              ))}
            </div>
          </section>

          <section aria-label={tt(lang, "indices.textAria", "Índice de texto")} className="border-2 border-foreground bg-card p-4">
            <div className="font-tech text-[10px] uppercase tracking-widest text-neon">{tt(lang, "indices.textTitle", "Índice de texto · BM25")}</div>
            <div className="mt-3 grid grid-cols-2 gap-3 md:grid-cols-4">
              {textIndexTiles(snapshot, lang).map((t) => (
                <Tile key={t.key} t={t} lang={lang} />
              ))}
            </div>
          </section>

          <section aria-label="WAL" className="border-2 border-foreground bg-card p-4">
            <div className="font-tech text-[10px] uppercase tracking-widest text-neon">WAL</div>
            <div className="mt-3 grid grid-cols-2 gap-3 md:grid-cols-4">
              {walTiles(snapshot, lang).map((t) => (
                <Tile key={t.key} t={t} lang={lang} />
              ))}
            </div>
          </section>
        </>
      )}

      {/* Gaps documentados — nunca inventar métricas en la UI. */}
      <section aria-label={tt(lang, "indices.gapsAria", "Métricas no expuestas por el core")} className="border-2 border-dashed border-muted-foreground p-4">
        <div className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
          {tt(lang, "indices.gapsTitle", "no expuesto por el core · follow-up")}
        </div>
        <ul className="mt-2 space-y-1">
          {(lang === "es" ? CORE_GAPS : coreGaps(lang)).map((g) => (
            <li key={g.label} className="font-tech text-[11px] text-muted-foreground">
              <span className="text-foreground">{g.label}</span> — {g.detail}
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
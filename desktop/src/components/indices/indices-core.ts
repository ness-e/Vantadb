// ÍNDICES lens pure logic (FEAT-02). All formatting/aggregation lives here so
// node:test covers it without React; IndicesLens is a thin renderer.
//
// "No mentir en UI": every tile maps to a field the core really exposes in
// `VantaOperationalMetrics` (src/sdk/types.rs) / `VantaNamespaceStats`. Metrics
// the core does NOT expose (dims, live LSM/WAL status) render as "—" and are
// listed in CORE_GAPS — never invented numbers.
import type { NamespaceStatsMap, OperationalMetrics } from "../../vanta";
// UX-15: fmtBytes compartido (era local aquí) — import + re-export para uso
// interno y para no romper indices-core.test.ts (que lo importa de este módulo).
// Extensión .ts explícita: node --test (strip-types ESM) no resuelve sin ella.
import { fmtBytes } from "../../lib/format.ts";
export { fmtBytes };

export function fmtCount(n: number): string {
  if (n >= 1e9) return `${(n / 1e9).toFixed(2)}B`;
  if (n >= 1e6) return `${(n / 1e6).toFixed(1)}M`;
  if (n >= 1e3) return `${(n / 1e3).toFixed(1)}K`;
  return `${n}`;
}

/** One namespace row: count + optional expiry buckets + bar width (0–100,
 * relative to the largest namespace — Cleveland–McGill P7: length primary). */
export interface NamespaceBar {
  name: string;
  count: number;
  /** null when the transport has no per-namespace stats (WASM list fallback). */
  expiringSoon: number | null;
  expired: number | null;
  widthPct: number;
}

function withWidths(bars: Omit<NamespaceBar, "widthPct">[]): NamespaceBar[] {
  const max = Math.max(0, ...bars.map((b) => b.count));
  return bars.map((b) => ({ ...b, widthPct: max > 0 ? (b.count / max) * 100 : 0 }));
}

/** Bars from the real per-namespace stats endpoint (native/HTTP). */
export function namespaceBars(stats: NamespaceStatsMap): NamespaceBar[] {
  return withWidths(
    Object.entries(stats)
      .map(([name, s]) => ({
        name,
        count: s.count,
        expiringSoon: s.expiring_soon,
        expired: s.expired,
      }))
      .sort((a, b) => b.count - a.count),
  );
}

/** Bars from a client-side list() count (WASM fallback: no expiry buckets). */
export function namespaceBarsFromCounts(counts: Record<string, number>): NamespaceBar[] {
  return withWidths(
    Object.entries(counts)
      .map(([name, count]) => ({ name, count, expiringSoon: null, expired: null }))
      .sort((a, b) => b.count - a.count),
  );
}

export interface IndexTile {
  key: string;
  label: string;
  value: string;
  muted?: string;
  /** true when the value is a documented core gap rendered as "—". */
  gap?: boolean;
}

/** HNSW/vector index tiles. `dims` is NOT exposed by the core — the tile
 * renders "—" and the gap is listed in CORE_GAPS (never invented). */
export function vectorIndexTiles(m: OperationalMetrics): IndexTile[] {
  return [
    { key: "hnsw_nodes", label: "Nodos HNSW", value: fmtCount(m.hnsw_nodes_count) },
    { key: "hnsw_bytes", label: "HNSW lógico", value: fmtBytes(m.hnsw_logical_bytes) },
    { key: "ann_rebuild", label: "Rebuild ANN", value: fmtCount(m.ann_rebuild_ms) + "ms", muted: "último rebuild" },
    { key: "dims", label: "Dimensionalidad", value: "—", gap: true, muted: "gap: el core no la expone" },
  ];
}

/** BM25 text index tiles. */
export function textIndexTiles(m: OperationalMetrics): IndexTile[] {
  return [
    { key: "postings", label: "Postings", value: fmtCount(m.text_postings_written) },
    { key: "queries", label: "Queries BM25", value: fmtCount(m.text_lexical_queries) },
    { key: "candidates", label: "Candidatos", value: fmtCount(m.text_candidates_scored) },
    { key: "repairs", label: "Repairs", value: fmtCount(m.text_index_repairs) },
  ];
}

/** WAL tiles. Only startup-replay counters exist — no live WAL status (gap). */
export function walTiles(m: OperationalMetrics): IndexTile[] {
  return [
    {
      key: "wal_replay",
      label: "Replay de arranque",
      value: fmtCount(m.wal_records_replayed),
      muted: `${m.wal_replay_ms}ms`,
    },
  ];
}

/** Métricas que el core NO expone hoy — documentadas, no inventadas (patrón
 * "no mentir en UI"). Follow-up: agregar campos a `VantaOperationalMetrics`. */
export const CORE_GAPS: { label: string; detail: string }[] = [
  {
    label: "dims",
    detail: "dimensión del índice vectorial — VantaOperationalMetrics no la expone",
  },
  {
    label: "LSM/WAL status",
    detail: "estado en vivo del storage — solo existen wal_replay_ms / wal_records_replayed de arranque",
  },
];
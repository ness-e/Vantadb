// retrieval-core.ts (VS-13): lógica pura de la lente RETRIEVAL.
//
// Descompone el score de un hit en segmentos apilados HONESTOS: el wire del
// bridge (VS-CORE-03) NO trae `fusion_report` — solo ranks por rama. El core
// calcula el score RRF como suma de contribuciones `1/(RRF_K + rank)`
// (src/planner.rs:25 `RRF_K = 60.0`, :174 contribución con rank 0-based; el
// wire expone ranks 1-based vía `rank_map` (src/sdk/search/debug.rs:23) →
// contribución = 1/(RRF_K + r_wire)). Con los ranks se reconstruye la
// contribución EXACTA de cada rama — no se inventa nada (Riesgo plan:159).
//
// Autónomo (sin imports) a propósito: corre en node puro para el self-check.
//
// Segmentos:
//   text   — contribución RRF de la rama BM25/texto (1/(RRF_K+rrf_text_rank))
//   vector — contribución RRF de la rama HNSW/vector (1/(RRF_K+rrf_vector_rank))
//   rrf    — residuo del score que no explican las ramas (fuse_rrf_many con
//            sparse o clamping): max(0, score − text − vector). 0 en híbrido 2-ramas.

export const RRF_K = 60;

/** Shape mínimo del explanation del wire (mirror de `ExplanationHit`). */
export interface ExplanationLike {
  score: number;
  rrf_text_rank?: number | null;
  rrf_vector_rank?: number | null;
  bm25_terms?: Array<{ token: string; contribution: number }>;
}

export interface ScoreSegment {
  key: "text" | "vector" | "rrf";
  label: string;
  /** Contribución RRF exacta (escala del score del hit). */
  value: number;
  /** Porcentaje del ancho de barra (0..100), normalizado contra maxScore. */
  widthPct: number;
}

export interface SegmentBreakdown {
  /** Score total del hit (escala RRF). */
  score: number;
  /** Suma de las contribuciones de rama (text + vector). */
  ramaSum: number;
  /** Segmentos con widthPct ya calculado; [] si no hay explanation. */
  segments: ScoreSegment[];
  /** true si el explanation estaba ausente (sin desglose). */
  missing: boolean;
}

/** Contribución RRF de un rank 1-based del wire. */
export function rrfContribution(rank: number | null | undefined): number {
  if (rank == null || rank < 1) return 0;
  return 1 / (RRF_K + rank);
}

/**
 * Descompone el explanation de un hit en segmentos apilados.
 * - `explanation` ausente (search sin explain o backend sin soporte) → segmentos
 *   vacíos, `missing: true` — el componente pinta barra de score sola, no crashea.
 * - `maxScore` es el mayor score del conjunto: normaliza el ancho para que todas
 *   las barras compartan escala. Si es 0/negativo, cae a 1 (anchos 0).
 */
export function computeSegments(
  explanation: ExplanationLike | null | undefined,
  maxScore: number,
): SegmentBreakdown {
  if (!explanation) {
    return { score: 0, ramaSum: 0, segments: [], missing: true };
  }
  const text = rrfContribution(explanation.rrf_text_rank);
  const vector = rrfContribution(explanation.rrf_vector_rank);
  // Umbral de ruido de punto flotante: (a+b)-a-b da ~1e-17 que NO es fusión.
  const rawRrf = explanation.score - text - vector;
  const rrf = rawRrf > 1e-9 ? rawRrf : 0;
  const scale = maxScore > 0 ? maxScore : 1;

  const segments: ScoreSegment[] = [];
  if (text > 0) {
    segments.push({
      key: "text",
      label: "texto (BM25)",
      value: text,
      widthPct: Math.min(100, (text / scale) * 100),
    });
  }
  if (vector > 0) {
    segments.push({
      key: "vector",
      label: "vector (HNSW)",
      value: vector,
      widthPct: Math.min(100, (vector / scale) * 100),
    });
  }
  if (rrf > 0) {
    segments.push({
      key: "rrf",
      label: "fusión RRF",
      value: rrf,
      widthPct: Math.min(100, (rrf / scale) * 100),
    });
  }

  return { score: explanation.score, ramaSum: text + vector, segments, missing: false };
}

// ── FEAT-01: slider de pesos híbridos (client-side, sin tocar core) ───────────
//
// El core fusiona SIEMPRE con RRF plano (src/planner.rs RRF_K=60) — el wire
// `VantaMemorySearchRequest` no acepta pesos (verificado FEAT-01 discovery).
// Pero el core expone el rank por rama de cada hit (`rrf_text_rank` /
// `rrf_vector_rank`), así que el peso se aplica client-side sobre los
// candidatos ya fusionados: es un weighted RRF, la generalización estándar de
// la fusión del core. α=0.5 reproduce EXACTAMENTE el orden del RRF del core
// (misma fórmula, factor escalar 1/2); α=0 ordena por texto puro; α=1 por
// vector puro. El conjunto de candidatos NO cambia — lo fija el core — solo el
// orden y el score. Gap honesto: pesos reales en core = follow-up (task core
// aditiva, patrón VS-CORE-*).

/** Slider 0..100 → peso α ∈ [0,1]. 0 = BM25 puro, 1 = vector puro, 0.5 = RRF. */
export function weightFromSlider(v: number): number {
  if (!Number.isFinite(v)) return 0.5;
  return Math.min(1, Math.max(0, v / 100));
}

/** Score ponderado de un hit: (1−α)·rrf(texto) + α·rrf(vector). La rama
 * ausente (rank null) contribuye 0. Con α=0.5 es 1/2 del score RRF del core
 * (mismo orden). */
export function weightedScore(
  explanation: ExplanationLike | null | undefined,
  alpha: number,
): number {
  if (!explanation) return 0;
  const text = rrfContribution(explanation.rrf_text_rank);
  const vector = rrfContribution(explanation.rrf_vector_rank);
  return (1 - alpha) * text + alpha * vector;
}

/** Shape mínimo para re-rank (un hit con score + explanation opcional). */
export interface WeightedHit {
  score: number;
  explanation?: ExplanationLike | null;
}

/** Re-ordena los candidatos por peso híbrido, sin mutar el input (copia con
 * `score` = score ponderado). Orden descendente; empates conservan el orden
 * previo (sort estable). */
export function rerankByWeight<T extends WeightedHit>(hits: T[], alpha: number): T[] {
  return hits
    .map((h) => ({ h, w: weightedScore(h.explanation, alpha) }))
    .sort((a, b) => b.w - a.w)
    .map(({ h, w }) => ({ ...h, score: w }));
}

/** Segmentos ponderados para ScoreBars con slider activo: la rama dominada por
 * el peso encoge, la dominante crece. text=(1−α)·c, vector=α·c; el residuo rrf
 * es 0 por construcción (2 ramas, sin sparse en el wire del desktop). */
export function computeSegmentsWeighted(
  explanation: ExplanationLike | null | undefined,
  maxScore: number,
  alpha: number,
): SegmentBreakdown {
  if (!explanation) {
    return { score: 0, ramaSum: 0, segments: [], missing: true };
  }
  const text = (1 - alpha) * rrfContribution(explanation.rrf_text_rank);
  const vector = alpha * rrfContribution(explanation.rrf_vector_rank);
  const scale = maxScore > 0 ? maxScore : 1;

  const segments: ScoreSegment[] = [];
  if (text > 0) {
    segments.push({
      key: "text",
      label: "texto (BM25)",
      value: text,
      widthPct: Math.min(100, (text / scale) * 100),
    });
  }
  if (vector > 0) {
    segments.push({
      key: "vector",
      label: "vector (HNSW)",
      value: vector,
      widthPct: Math.min(100, (vector / scale) * 100),
    });
  }

  return { score: text + vector, ramaSum: text + vector, segments, missing: false };
}
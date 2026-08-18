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
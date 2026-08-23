// FEAT-03a CONSOLIDAR pure logic (node --test, no React).
// Detección de pares candidatos (duplicados/superados) por similitud textual
// (search kNN) + merge de metadata `superseded_by` + helpers de diff.
// Autónoma de React/transporte — la lente (ConsolidateLens) solo la orquesta.

export interface PairRecord {
  id: string;
  namespace: string;
  text: string;
  metadata?: Record<string, unknown>;
}

export interface SearchHitLike {
  id: string;
  namespace: string;
  text: string;
  metadata?: Record<string, unknown>;
  score: number;
}

export interface CandidatePair {
  a: PairRecord;
  b: PairRecord;
  /** Relevance del hit que formó el par (backend-defined, escala RRF: ~1/61 top-1). */
  score: number;
}

export const SUPERSEDED_BY_KEY = "superseded_by";

// Límites del run de detección — pisos honestos para una lente local, no
// garantías de exhaustividad (ponytail: ceilings conocidos; subir si una DB
// real los golpea).
export const MIN_PAIR_SCORE = 0.01; // piso de score bruto (RRF) para considerar un par
export const MAX_PAIRS = 25; // pares mostrados por run
export const MAX_RECORDS = 200; // registros escaneados por run
export const MAX_QUERIES = 50; // búsquedas por registro por run
export const TOP_K = 5; // hits pedidos por búsqueda

/** Clave canónica de par: orden lexicográfico de ids, independiente de la
 * dirección de la consulta (a→b y b→a colapsan al mismo par). */
export function pairKey(aId: string, bId: string): string {
  return aId <= bId ? `${aId}\u0000${bId}` : `${bId}\u0000${aId}`;
}

/** Convierte los hits por registro en pares candidatos deduplicados: excluye
 * self-hits y registros no cargados (otro namespace), colapsa la dirección
 * inversa quedándose con el mejor score, ordena desc y corta por maxPairs. */
export function buildCandidatePairs(
  records: PairRecord[],
  hitsByKey: Map<string, SearchHitLike[]>,
  opts: { minScore?: number; maxPairs?: number } = {},
): CandidatePair[] {
  const minScore = opts.minScore ?? MIN_PAIR_SCORE;
  const maxPairs = opts.maxPairs ?? MAX_PAIRS;
  const byId = new Map(records.map((r) => [r.id, r]));
  const best = new Map<string, CandidatePair>();
  for (const rec of records) {
    for (const hit of hitsByKey.get(rec.id) ?? []) {
      if (hit.id === rec.id) continue;
      if (hit.score < minScore) continue;
      const other = byId.get(hit.id);
      if (!other) continue;
      const k = pairKey(rec.id, hit.id);
      const prev = best.get(k);
      if (!prev || hit.score > prev.score) {
        best.set(k, { a: rec, b: other, score: hit.score });
      }
    }
  }
  return [...best.values()].sort((x, y) => y.score - x.score).slice(0, maxPairs);
}

/** Metadata del registro superado: preserva las claves existentes y agrega
 * `superseded_by = <id vigente>` (la clave es metadata de usuario en el core). */
export function mergeSuperseded(
  metadata: Record<string, unknown> | undefined,
  supersededById: string,
): Record<string, unknown> {
  return { ...(metadata ?? {}), [SUPERSEDED_BY_KEY]: supersededById };
}

/** Id vigente que supera a este registro, o null si no está marcado. */
export function supersededBy(metadata: Record<string, unknown> | undefined): string | null {
  const v = metadata?.[SUPERSEDED_BY_KEY];
  return typeof v === "string" && v.length > 0 ? v : null;
}

/** Cuenta registros ya marcados como superados (para el resumen de la lente). */
export function countSuperseded(records: PairRecord[]): number {
  return records.filter((r) => supersededBy(r.metadata) !== null).length;
}

/** Similitud visual del par: pct relativo al mejor score del run (barra), con
 * clamp 0..100. maxScore 0 → 0 (run vacío, safe). */
export function fmtSim(score: number, maxScore: number): { pct: number; label: string } {
  const raw = maxScore > 0 ? Math.round((score / maxScore) * 100) : 0;
  const pct = Math.min(100, Math.max(0, raw));
  return { pct, label: `${pct}%` };
}

// ── MEM-58: consolidación real vía context engine ──────────────────────────

/** Outcome del comando IPC `vanta_context_assemble` (mirror de
 * `IntegratedContext` de vanta-memory — ya serde snake_case). */
export interface AssembledContext {
  messages: { role: string; content: string }[];
  report: {
    mode: string;
    msgs_conserved: number;
    msgs_before: number;
    tokens_before: number;
    tokens_after: number;
  };
  mmd_injected: boolean;
  recall_injected: boolean;
}

/** Budget por defecto del run real (tokens). */
export const ASSEMBLE_BUDGET_TOKENS = 1200;

/** Registros del namespace → chat history para el engine real: cada registro
 * es un turno user con su id preservado (el cursor guard puede referenciarlo). */
export function toHistory(records: PairRecord[]): { role: "user"; content: string; id?: string }[] {
  return records.map((r) => ({ role: "user" as const, content: r.text, id: r.id }));
}

/** Resumen humano de un run real: "3/5 msgs · 900→250 tokens · mild · recall ✓". */
export function formatAssembleReport(out: AssembledContext): string {
  const parts = [
    `${out.report.msgs_conserved}/${out.report.msgs_before} msgs`,
    `${out.report.tokens_before}→${out.report.tokens_after} tokens`,
    out.report.mode,
  ];
  if (out.mmd_injected) parts.push("mmd ✓");
  if (out.recall_injected) parts.push("recall ✓");
  return parts.join(" · ");
}
// FEAT-03a Lente CONSOLIDAR (D16 (a) UI-only): detecta candidatos duplicados/
// superados por similitud textual (search kNN sobre el texto de cada registro),
// muestra el diff visible entre pares y escribe `metadata.superseded_by` en el
// registro superado vía vanta_put. La lógica pura vive en consolidate-core.ts
// (node-testable); esta lente solo orquesta el bridge (funciona en los 3
// transports: Tauri/REST/WASM — mismos comandos vanta_search/vanta_put/vanta_list).
import { useState } from "react";
import {
  listPage,
  search,
  vantaErrorMessage,
  vantaPut,
  type MemoryRecord,
} from "../../vanta";
import {
  buildCandidatePairs,
  countSuperseded,
  fmtSim,
  MAX_PAIRS,
  MAX_QUERIES,
  MAX_RECORDS,
  mergeSuperseded,
  SUPERSEDED_BY_KEY,
  supersededBy,
  TOP_K,
  type CandidatePair,
  type PairRecord,
} from "./consolidate-core.ts";

const PAGE_SIZE = 100;

function RecordCard({ rec, score, maxScore }: { rec: PairRecord; score: number; maxScore: number }) {
  const sup = supersededBy(rec.metadata);
  const sim = fmtSim(score, maxScore);
  const metaEntries = Object.entries(rec.metadata ?? {}).filter(([k]) => k !== SUPERSEDED_BY_KEY);
  return (
    <div className="flex min-w-0 flex-1 flex-col border-2 border-foreground bg-card">
      <div className="flex items-center justify-between gap-2 border-b-2 border-foreground px-2 py-1">
        <span className="truncate font-tech text-[10px] uppercase tracking-widest text-neon">
          {rec.namespace}/{rec.id}
        </span>
        <span className="shrink-0 font-tech text-[9px] text-muted-foreground" title={`score ${score.toFixed(4)}`}>
          {sim.label}
        </span>
      </div>
      <div className="h-1 w-full bg-muted">
        <div className="h-1 bg-neon" style={{ width: `${sim.pct}%` }} />
      </div>
      <p className="scroll-manga flex-1 whitespace-pre-wrap break-words px-2 py-2 text-xs leading-relaxed">
        {rec.text}
      </p>
      {metaEntries.length > 0 && (
        <div className="flex flex-wrap gap-1 border-t-2 border-foreground px-2 py-1">
          {metaEntries.map(([k, v]) => (
            <span key={k} className="border border-foreground px-1 font-tech text-[9px] text-muted-foreground">
              {k}: {typeof v === "object" ? JSON.stringify(v) : String(v)}
            </span>
          ))}
        </div>
      )}
      {sup && (
        <div className="border-t-2 border-foreground bg-foreground px-2 py-1 font-tech text-[9px] uppercase tracking-widest text-background">
          ✕ superado por {sup}
        </div>
      )}
    </div>
  );
}

export default function ConsolidateLens({
  active,
  activeName,
  onNotice,
  onError,
}: {
  active: boolean;
  activeName: string | null;
  onNotice: (msg: string) => void;
  onError: (msg: string) => void;
}) {
  const [ns, setNs] = useState("");
  const [detecting, setDetecting] = useState(false);
  const [pairs, setPairs] = useState<CandidatePair[] | null>(null);
  const [records, setRecords] = useState<PairRecord[] | null>(null);
  const [runInfo, setRunInfo] = useState("");

  const maxScore = pairs && pairs.length > 0 ? pairs[0].score : 0;
  const supersededCount = records ? countSuperseded(records) : 0;

  async function runDetection() {
    if (!active) {
      onError("Sin backend activo — conectá uno para operar");
      return;
    }
    const nsArg = ns.trim() || undefined;
    setDetecting(true);
    try {
      // 1. Cargar registros del namespace (paginado, con tope honesto).
      const all: MemoryRecord[] = [];
      let cursor: number | undefined;
      for (;;) {
        const page = await listPage({ namespace: nsArg, limit: PAGE_SIZE, cursor });
        all.push(...page.records);
        if (all.length >= MAX_RECORDS || page.next_cursor == null) break;
        cursor = page.next_cursor;
      }
      const recs: PairRecord[] = all
        .filter((r) => r.text.trim().length > 0)
        .map((r) => ({ id: r.id, namespace: r.namespace, text: r.text, metadata: r.metadata ?? {} }));

      // 2. Detección: search kNN con el texto de cada registro (top_k modesto).
      const hitsByKey = new Map<string, { id: string; namespace: string; text: string; metadata?: Record<string, unknown>; score: number }[]>();
      const scanned = Math.min(recs.length, MAX_QUERIES);
      for (let i = 0; i < scanned; i++) {
        const r = recs[i];
        const hits = await search({ query: r.text, top_k: TOP_K, namespace: nsArg });
        hitsByKey.set(
          r.id,
          hits.map((h) => ({ id: h.id, namespace: h.namespace, text: h.text, metadata: h.metadata, score: h.score })),
        );
      }

      const found = buildCandidatePairs(recs, hitsByKey);
      setRecords(recs);
      setPairs(found);
      setRunInfo(
        `${recs.length} registros · ${scanned} búsquedas · ${found.length} pares (top ${MAX_PAIRS}) · umbral score ${found.length > 0 ? "aplicado" : "sin pares"}`,
      );
      onNotice(
        found.length > 0
          ? `Consolidación: ${found.length} par(es) candidato(s) detectado(s)`
          : "Consolidación: sin pares candidatos sobre el umbral",
      );
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setDetecting(false);
    }
  }

  /** Marca `sup` como superado por `kept`: upsert con metadata mergeada (el
   * core reemplaza el registro — se reenvía el payload original). */
  async function markSuperseded(sup: PairRecord, kept: PairRecord) {
    try {
      await vantaPut({
        namespace: sup.namespace,
        key: sup.id,
        payload: sup.text,
        metadata: mergeSuperseded(sup.metadata, kept.id),
      });
      setRecords((prev) =>
        prev ? prev.map((r) => (r.id === sup.id ? { ...r, metadata: mergeSuperseded(r.metadata, kept.id) } : r)) : prev,
      );
      onNotice(`marcado ${sup.namespace}/${sup.id} → superado por ${kept.id}`);
    } catch (err) {
      onError(vantaErrorMessage(err));
    }
  }

  return (
    <section aria-label="Consolidación asistida" className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 border-b-4 border-foreground pb-2">
        <div className="flex items-center gap-2">
          <span className="text-neon">⇄</span>
          <h2 className="font-display text-2xl text-stencil">CONSOLIDAR</h2>
        </div>
        <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
          duplicados · superados · diff
        </span>
      </div>

      {/* Controles */}
      <div className="flex flex-wrap items-center gap-2 border-2 border-foreground bg-card p-3">
        <label className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground" htmlFor="cons-ns">
          namespace
        </label>
        <input
          id="cons-ns"
          value={ns}
          onChange={(e) => setNs(e.target.value)}
          placeholder={activeName ?? "default"}
          aria-label="Namespace a consolidar (vacío = todos)"
          className="w-40 border-2 border-foreground bg-background px-2 py-1 text-xs placeholder:text-muted-foreground"
        />
        <button
          type="button"
          onClick={runDetection}
          disabled={detecting}
          className="press border-2 border-foreground bg-background px-3 py-1 text-xs font-semibold disabled:opacity-50"
        >
          {detecting ? "detectando…" : "⛃ Detectar candidatos"}
        </button>
        {supersededCount > 0 && (
          <span className="font-tech text-[10px] uppercase tracking-widest text-neon">
            {supersededCount} marcado(s)
          </span>
        )}
        {runInfo && <span className="ml-auto font-tech text-[9px] text-muted-foreground">{runInfo}</span>}
      </div>

      {/* Estado sin backend */}
      {!active && (
        <div className="border-2 border-foreground bg-card p-6 text-center font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
          sin backend activo — conectá uno para operar
        </div>
      )}

      {/* Pares candidatos */}
      {pairs !== null && pairs.length === 0 && (
        <div className="border-2 border-foreground bg-card p-6 text-center font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
          sin pares candidatos sobre el umbral — probá otro namespace o ingerí duplicados primero
        </div>
      )}
      {pairs !== null && pairs.length > 0 && (
        <div className="space-y-4">
          {pairs.map((p) => (
            <article key={p.a.id + p.b.id} className="border-2 border-foreground bg-card">
              <div className="flex flex-wrap items-center justify-between gap-2 border-b-2 border-foreground bg-muted px-2 py-1">
                <span className="font-tech text-[9px] uppercase tracking-widest text-neon">
                  par · score {p.score.toFixed(4)} · similitud {fmtSim(p.score, maxScore).label}
                </span>
                <div className="flex gap-1">
                  <button
                    type="button"
                    onClick={() => markSuperseded(p.a, p.b)}
                    disabled={detecting}
                    className="press border-2 border-foreground bg-background px-2 py-0.5 font-tech text-[9px] disabled:opacity-50"
                    title={`Escribe metadata.superseded_by = ${p.b.id} en ${p.a.id}`}
                  >
                    → {p.a.id} superado por {p.b.id}
                  </button>
                  <button
                    type="button"
                    onClick={() => markSuperseded(p.b, p.a)}
                    disabled={detecting}
                    className="press border-2 border-foreground bg-background px-2 py-0.5 font-tech text-[9px] disabled:opacity-50"
                    title={`Escribe metadata.superseded_by = ${p.a.id} en ${p.b.id}`}
                  >
                    → {p.b.id} superado por {p.a.id}
                  </button>
                </div>
              </div>
              <div className="flex flex-col gap-2 p-2 sm:flex-row">
                <RecordCard rec={p.a} score={p.score} maxScore={maxScore} />
                <RecordCard rec={p.b} score={p.score} maxScore={maxScore} />
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
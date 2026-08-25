// RetrievalLens.tsx (VS-13): Lente RETRIEVAL — "¿por qué recuperó esto?".
//
// Barra de consulta (texto + vector-picker de registro existente + top-k +
// umbral) + filtros visuales por metadata (REUSA filters-core/FiltersBuilder
// de VS-07, no duplica) + resultados con desglose de score (ScoreBars) +
// "ver contexto" por resultado (vecino semántico vía search con el vector del
// registro + historial del audit vía auditEvents si está disponible).
//
// Estética manga/linocut (tokens VS-01). Sin charts lib — barras CSS.
import { FormEvent, lazy, Suspense, useEffect, useMemo, useRef, useState } from "react";
import type { RuleGroupType } from "react-querybuilder";
import {
  auditEvents,
  get,
  listPage,
  search,
  vantaErrorMessage,
  type AuditEvent,
  type MemoryRecord,
  type SearchResult,
} from "../../../vanta";
import {
  EMPTY_QUERY,
  evaluateQuery,
  inferMetaFields,
  toVantaMemoryFilter,
} from "../../search/filters-core";
import ScoreBars from "./ScoreBars";
import LensShell from "../../layout/LensShell";
import { fusionModeFromSlider } from "./retrieval-core";

// react-querybuilder (~200 kB) solo lo abre el panel de filtros → lazy igual
// que el shell (VS-07).
const FiltersBuilder = lazy(() => import("../../search/FiltersBuilder"));

interface Props {
  /** Registro semilla (P4: lente contextual desde el Inspector) — su vector
   * preselecciona el vector-picker si existe. */
  seed?: MemoryRecord | null;
  onNotice: (msg: string) => void;
  onError: (msg: string) => void;
  /** Abre un record en el Inspector (master-detail) — "ver contexto". */
  onOpenRecord?: (record: MemoryRecord, score: number | null) => void;
}

interface ResultRow extends SearchResult {
  /** Vecino semántico cacheado por hit (get → search con su vector). */
  context?: MemoryRecord | null;
}

export default function RetrievalLens({ seed, onNotice, onError, onOpenRecord }: Props) {
  // --- Consulta --------------------------------------------------------------
  const [textQuery, setTextQuery] = useState("");
  const [topK, setTopK] = useState(10);
  const [threshold, setThreshold] = useState(0);
  const [searching, setSearching] = useState(false);

  // Vector-picker: registros existentes con vector (para search por embedding).
  const [records, setRecords] = useState<MemoryRecord[]>([]);
  const [pickedId, setPickedId] = useState<string>("");

  // --- Filtros visuales por metadata (VS-07, REUSADO) -------------------------
  const [ruleGroup, setRuleGroup] = useState<RuleGroupType>(EMPTY_QUERY);
  const [showFilters, setShowFilters] = useState(false);

  const [results, setResults] = useState<ResultRow[] | null>(null);
  // Slider de modo de fusión (DESKTOP-35): 0 = BM25 puro, 50 = RRF híbrido,
  // 100 = vector puro. Se envía como `search_profile` en el request — los
  // resultados son SIEMPRE los del servidor (idénticos a su explain). El core
  // no soporta pesos intermedios: slider discreto con stops 0/50/100.
  const [weight, setWeight] = useState(50);

  // Carga registros con vector para el picker (una página; ponytail: 200 es
  // suficiente para una DB local de studio — swap a cursor si se escala).
  useEffect(() => {
    let alive = true;
    listPage({ limit: 200 })
      .then((page) => {
        if (!alive) return;
        const withVector = page.records.filter((r) => r.vector && r.vector.length > 0);
        setRecords(withVector);
        // Seed contextual (P4): si el registro seleccionado tiene vector, lo
        // preselecciona; si no, el primer record con vector.
        if (seed?.vector && seed.vector.length > 0) {
          setPickedId(`${seed.namespace}:${seed.id}`);
        } else if (withVector.length > 0) {
          setPickedId(`${withVector[0].namespace}:${withVector[0].id}`);
        }
      })
      .catch((err) => onError(vantaErrorMessage(err)));
    return () => {
      alive = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [seed?.id, seed?.namespace]);

  // Re-sincroniza el seed cuando cambia el registro seleccionado.
  useEffect(() => {
    if (seed?.vector && seed.vector.length > 0) {
      setPickedId(`${seed.namespace}:${seed.id}`);
    }
  }, [seed?.id, seed?.namespace, seed?.vector]);

  const pickedRecord = useMemo(
    () => records.find((r) => `${r.namespace}:${r.id}` === pickedId) ?? null,
    [records, pickedId],
  );

  const filterActive = ruleGroup.rules.length > 0;
  const filterFields = useMemo(() => (results ? inferMetaFields(results) : []), [results]);

  // --- DESKTOP-35: sin re-rank client-side — el orden viene del servidor.
  const visibleResults = useMemo(() => {
    if (!results) return null;
    let out = filterActive
      ? results.filter((r) => evaluateQuery(ruleGroup, r.metadata ?? {}))
      : results;
    if (threshold > 0) out = out.filter((r) => r.score >= threshold);
    return out;
  }, [results, ruleGroup, filterActive, threshold]);

  // Escala común de barras sobre el ranking del servidor.
  const displayMax = useMemo(() => {
    if (!visibleResults || visibleResults.length === 0) return 1;
    const m = Math.max(...visibleResults.map((r) => r.score));
    return m > 0 ? m : 1;
  }, [visibleResults]);

  // --- Búsqueda con explain ---------------------------------------------------
  async function runSearch(e?: FormEvent) {
    e?.preventDefault();
    const q = textQuery.trim();
    if (!q && !pickedRecord) {
      onNotice("Escribí una query o elegí un registro vectorial");
      return;
    }
    setSearching(true);
    try {
      // DESKTOP-35: el slider fija el modo de fusión server-side (MEM-01).
      // Los resultados son los del servidor — mismos que su explain.
      const hits = await search({
        query: q,
        embedding: pickedRecord?.vector ?? undefined,
        top_k: topK,
        explain: true,
        search_profile: fusionModeFromSlider(weight),
      });
      setResults(hits);
    } catch (err) {
      onError(vantaErrorMessage(err));
      setResults(null);
    } finally {
      setSearching(false);
    }
  }

  function clearResults() {
    setResults(null);
  }

  // Re-ejecuta la búsqueda al cambiar el slider si ya hay resultados
  // (antes el re-rank era instantáneo client-side; ahora el server decide).
  const skipWeightEffect = useRef(true);
  useEffect(() => {
    if (skipWeightEffect.current) {
      skipWeightEffect.current = false;
      return;
    }
    if (results === null || searching) return;
    void runSearch();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [weight]);

  // --- "Ver contexto" (e): vecino semántico + historial del audit --------------
  const [contextKey, setContextKey] = useState<string | null>(null);
  const [contextLoading, setContextLoading] = useState(false);
  const [context, setContext] = useState<{
    record: MemoryRecord | null;
    neighbors: SearchResult[];
    audit: AuditEvent[];
    auditErr: string | null;
  } | null>(null);

  async function toggleContext(r: ResultRow) {
    const key = `${r.namespace}:${r.id}`;
    if (contextKey === key) {
      setContextKey(null);
      return;
    }
    setContextKey(key);
    setContextLoading(true);
    setContext(null);
    try {
      // Vecino semántico: get completo (trae vector) → search con su embedding.
      let record: MemoryRecord | null = null;
      let neighbors: SearchResult[] = [];
      try {
        record = await get(r.id, r.namespace);
        if (record.vector && record.vector.length > 0) {
          neighbors = await search({
            query: "",
            embedding: record.vector,
            top_k: 5,
            explain: false,
          });
        }
      } catch (err) {
        onError(vantaErrorMessage(err));
      }
      // Historial del audit (VS-12): filtra client-side por key (el API no filtra).
      let audit: AuditEvent[] = [];
      let auditErr: string | null = null;
      try {
        const page = await auditEvents({ namespace: r.namespace, limit: 100 });
        audit = page.events.filter((ev) => ev.key === r.id);
      } catch (err) {
        auditErr = vantaErrorMessage(err);
      }
      setContext({ record, neighbors, audit, auditErr });
    } finally {
      setContextLoading(false);
    }
  }

  return (
    <section className="press-lg border-4 border-foreground bg-card" aria-label="Lente RETRIEVAL">
      {/* Header (UX-01: LensShell compartido) */}
      <div className="p-4">
        <LensShell
          title="RETRIEVAL"
          // UX-15: el header hablaba jerga ("explain on", "fusión server-side
          // (MEM-01)", "resultados = explain del server") — texto de usuario.
          meta="¿por qué recuperó esto? · desglose del score"
          subtitle={`búsqueda híbrida BM25 + vector — el slider elige el modo de fusión, que decide el servidor: ${
            weight === 0 ? "solo texto (BM25)" : weight === 100 ? "solo vector (HNSW)" : "híbrido (RRF)"
          }`}
        />
      </div>

      {/* Barra de consulta (b) */}
      <form onSubmit={runSearch} className="space-y-3 border-b-4 border-foreground p-4">
        <div className="flex flex-wrap items-end gap-3">
          <label className="flex min-w-[220px] flex-1 flex-col gap-1">
            <span className="font-tech text-[10px] uppercase tracking-widest text-neon">
              query de texto
            </span>
            <input
              type="search"
              value={textQuery}
              onChange={(e) => setTextQuery(e.target.value)}
              placeholder="¿qué buscar? (BM25 + HNSW híbrido)"
              className="w-full border-2 border-foreground bg-background px-3 py-1.5 text-sm placeholder:text-muted-foreground"
            />
          </label>

          <label className="flex min-w-[220px] flex-1 flex-col gap-1">
            <span className="font-tech text-[10px] uppercase tracking-widest text-neon">
              o vector de registro
            </span>
            <select
              value={pickedId}
              onChange={(e) => setPickedId(e.target.value)}
              className="w-full border-2 border-foreground bg-background px-2 py-1.5 text-sm"
              title="Usa el embedding de un registro existente como query vectorial"
            >
              <option value="">— ninguno —</option>
              {records.map((r) => (
                <option key={`${r.namespace}:${r.id}`} value={`${r.namespace}:${r.id}`}>
                  {r.namespace}:{r.id}
                </option>
              ))}
            </select>
          </label>

          <label className="flex w-20 flex-col gap-1">
            <span className="font-tech text-[10px] uppercase tracking-widest text-neon">top-k</span>
            <input
              type="number"
              min={1}
              max={100}
              value={topK}
              onChange={(e) => setTopK(Number(e.target.value) || 10)}
              className="w-full border-2 border-foreground bg-background px-2 py-1.5 text-sm"
            />
          </label>

          <label className="flex w-24 flex-col gap-1">
            <span className="font-tech text-[10px] uppercase tracking-widest text-neon">umbral</span>
            <input
              type="number"
              min={0}
              step={0.001}
              value={threshold}
              onChange={(e) => setThreshold(Number(e.target.value) || 0)}
              className="w-full border-2 border-foreground bg-background px-2 py-1.5 text-sm"
              title="Descarta hits con score menor (0 = sin umbral)"
            />
          </label>

          <button
            type="submit"
            disabled={searching}
            className="btn-neon-glow border-2 border-foreground bg-neon px-4 py-1.5 text-xs font-bold text-background"
          >
            {searching ? "…" : "▸ EXPLICAR"}
          </button>
          {results && (
            <button
              type="button"
              onClick={clearResults}
              className="press border-2 border-foreground bg-background px-3 py-1.5 text-xs"
            >
              ✕ limpiar
            </button>
          )}
        </div>

        {/* Slider de modo de fusión (DESKTOP-35): 0=BM25 puro · 50=RRF · 100=vector puro */}
        <label
          className="flex flex-col gap-1 border-2 border-foreground bg-background p-2"
          title="Modo de fusión server-side (MEM-01): 0 = solo texto (BM25), 50 = RRF híbrido, 100 = solo vector. El cambio re-ejecuta la búsqueda en el servidor."
        >
          <span className="flex items-baseline justify-between font-tech text-[10px] uppercase tracking-widest text-neon">
            <span>modo de fusión · BM25 ⟷ vector</span>
            <span className="text-foreground">
              {weight === 0 ? "BM25 puro" : weight === 100 ? "vector puro" : "RRF híbrido"}
            </span>
          </span>
          <input
            type="range"
            min={0}
            max={100}
            step={50}
            value={weight}
            onChange={(e) => setWeight(Number(e.target.value))}
            className="vanta-slider w-full"
            aria-label="Modo de fusión: 0 = solo texto (BM25), 50 = RRF híbrido, 100 = solo vector"
          />
        </label>

        {/* Filtros visuales por metadata (c) — REUSO VS-07 */}
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setShowFilters((v) => !v)}
            aria-pressed={showFilters}
            className={`press border-2 border-foreground px-2.5 py-1 text-[10px] font-semibold ${
              filterActive ? "bg-neon text-background" : "bg-background"
            }`}
            title="Filtros compuestos por metadata (AND/OR, sin JSON)"
          >
            ⧩ FILTROS{filterActive ? ` (${toVantaMemoryFilter(ruleGroup).length})` : ""}
          </button>
          {filterActive && (
            <button
              type="button"
              onClick={() => setRuleGroup(EMPTY_QUERY)}
              className="press border-2 border-foreground bg-background px-2 py-0.5 text-[10px] font-semibold"
            >
              ✕ limpiar filtros
            </button>
          )}
          {pickedRecord && (
            <span className="font-tech text-[10px] text-muted-foreground">
              vector: {pickedRecord.namespace}:{pickedRecord.id} · {pickedRecord.vector?.length}d
            </span>
          )}
        </div>

        {showFilters && (
          <div className="border-2 border-dashed border-foreground bg-background p-3">
            {filterFields.length === 0 ? (
              <p className="font-tech text-[11px] text-muted-foreground">
                Sin campos de metadata — ejecutá una búsqueda para inferir tipos
                (string/int/float/bool/datetime).
              </p>
            ) : (
              <Suspense
                fallback={<p className="font-tech text-[11px] text-muted-foreground">Cargando builder…</p>}
              >
                <FiltersBuilder fields={filterFields} query={ruleGroup} onChange={setRuleGroup} />
              </Suspense>
            )}
          </div>
        )}
      </form>

      {/* Resultados (d,e,f) */}
      <div className="p-4">
        {results === null ? (
          <p className="py-6 text-center font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
            {searching
              ? "buscando…"
              : "ejecutá una búsqueda para ver el desglose de score por hit"}
          </p>
        ) : results.length === 0 ? (
          <p className="py-6 text-center font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
            sin resultados — probá otra query o subí el top-k
          </p>
        ) : visibleResults === null || visibleResults.length === 0 ? (
          <p className="py-6 text-center font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
            todos los hits cayeron bajo el umbral {threshold.toFixed(3)} o el filtro de metadata
          </p>
        ) : (
          <ol className="space-y-3">
            {visibleResults.map((r) => {
              const key = `${r.namespace}:${r.id}`;
              const open = contextKey === key;
              return (
                <li
                  key={key}
                  className="border-2 border-foreground bg-background p-3"
                  aria-label={`Resultado ${r.id}`}
                >
                  <div className="flex items-baseline justify-between gap-2">
                    <div className="min-w-0">
                      <code className="truncate font-tech text-sm">{r.id}</code>
                      <span className="ml-2 border-2 border-foreground bg-background px-1.5 py-0.5 font-tech text-[10px]">
                        {r.namespace}
                      </span>
                    </div>
                    <span className="shrink-0 font-tech text-[10px] text-muted-foreground">
                      {r.explanation?.matched_tokens?.length
                        ? `tokens: ${r.explanation.matched_tokens.join(" ")}`
                        : "sin tokens (vector-only)"}
                    </span>
                  </div>
                  <p className="mt-1 line-clamp-2 text-[13px] opacity-80">{r.text}</p>

                  {/* Barras apiladas: desglose de score (d) + número (f) */}
                  <div className="mt-2">
                    <ScoreBars
                      explanation={r.explanation}
                      score={r.score}
                      maxScore={displayMax}
                    />
                  </div>

                  <div className="mt-2 flex items-center gap-2">
                    {onOpenRecord && (
                      <button
                        type="button"
                        onClick={() => onOpenRecord(
                          { id: r.id, namespace: r.namespace, text: r.text, metadata: r.metadata } as MemoryRecord,
                          r.score,
                        )}
                        className="press border-2 border-foreground bg-background px-2 py-0.5 text-[10px] font-semibold"
                      >
                        ▸ INSPECTOR
                      </button>
                    )}
                    <button
                      type="button"
                      onClick={() => void toggleContext(r)}
                      aria-expanded={open}
                      className={`press border-2 border-foreground px-2 py-0.5 text-[10px] font-semibold ${
                        open ? "bg-neon text-background" : "bg-background"
                      }`}
                    >
                      {open ? "▾ ocultar contexto" : "▸ ver contexto"}
                    </button>
                    {r.explanation?.snippet && (
                      <span className="truncate font-tech text-[10px] text-muted-foreground">
                        «{r.explanation.snippet}»
                      </span>
                    )}
                  </div>

                  {open && (
                    <div className="mt-2 border-t-2 border-dashed border-foreground pt-2">
                      {contextLoading ? (
                        <p className="font-tech text-[10px] text-muted-foreground">cargando contexto…</p>
                      ) : context ? (
                        <div className="space-y-2 font-tech text-[11px]">
                          {/* Vecino semántico (e) */}
                          <div>
                            <div className="font-tech text-[10px] uppercase tracking-widest text-neon">
                              vecino semántico
                            </div>
                            {context.record?.vector && context.record.vector.length > 0 ? (
                              context.neighbors.length > 0 ? (
                                <ul className="mt-1 space-y-0.5">
                                  {context.neighbors.map((n) => (
                                    <li key={`${n.namespace}:${n.id}`} className="flex items-baseline gap-2">
                                      <span className="w-12 shrink-0 text-right text-muted-foreground">
                                        {n.score.toFixed(3)}
                                      </span>
                                      <code className="truncate">{n.id}</code>
                                    </li>
                                  ))}
                                </ul>
                              ) : (
                                <p className="mt-1 text-muted-foreground">sin vecinos</p>
                              )
                            ) : (
                              <p className="mt-1 text-muted-foreground">
                                el registro no tiene vector propio — no hay vecinos semánticos
                              </p>
                            )}
                          </div>
                          {/* Historial del audit (e) */}
                          <div>
                            <div className="font-tech text-[10px] uppercase tracking-widest text-neon">
                              historial (audit)
                            </div>
                            {context.auditErr ? (
                              <p className="mt-1 text-muted-foreground">
                                {context.auditErr} — {context.auditErr.includes("no configurado") ? "activá audit_log_path en el backend" : ""}
                              </p>
                            ) : context.audit.length === 0 ? (
                              <p className="mt-1 text-muted-foreground">sin eventos para este key</p>
                            ) : (
                              <ul className="mt-1 space-y-0.5">
                                {context.audit.slice(0, 10).map((ev, i) => (
                                  <li key={`${ev.timestamp}-${i}`} className="flex items-baseline gap-2">
                                    <span className="shrink-0 text-muted-foreground">{ev.op}</span>
                                    <span className={`shrink-0 ${ev.outcome === "ok" ? "text-neon" : "text-foreground"}`}>
                                      {ev.outcome}
                                    </span>
                                    <span className="truncate opacity-70">{ev.timestamp}</span>
                                  </li>
                                ))}
                              </ul>
                            )}
                          </div>
                        </div>
                      ) : null}
                    </div>
                  )}
                </li>
              );
            })}
          </ol>
        )}
      </div>
    </section>
  );
}
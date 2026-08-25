// FEAT-03a CONSOLIDAR lente (DESKTOP-33): flujo completo detectar duplicados
// (kNN textual) → revisar lado a lado → merge campo a campo (pre-fill desde el
// dominante) o descartar el supersedido (papelera con undo / permanente, 2
// pasos) → acción batch sobre selección. Todo sin salir de la lente.
// El styling es Tailwind (DESKTOP-28).
import { useState } from "react";
import { TauriBackend, transport } from "../../transport";
import { undoStore } from "../../store/undo";
import {
  contextAssemble,
  get,
  listPage,
  remove,
  search,
  vantaErrorMessage,
  vantaPut,
  type MemoryRecord,
} from "../../vanta";
import {
  buildCandidatePairs,
  countSuperseded,
  defaultSources,
  fmtSim,
  formatAssembleReport,
  ASSEMBLE_BUDGET_TOKENS,
  MAX_PAIRS,
  MAX_QUERIES,
  MAX_RECORDS,
  mergeFields,
  mergeSuperseded,
  pairKey,
  SUPERSEDED_BY_KEY,
  supersededBy,
  toHistory,
  TOP_K,
  type CandidatePair,
  type FieldSource,
  type PairRecord,
} from "./consolidate-core.ts";
import ConfirmDiscard from "./ConfirmDiscard";
import LensShell from "../layout/LensShell";
import { ShieldCheck } from "lucide-react";

const PAGE_SIZE = 100;

const BTN =
  "press border-2 border-foreground bg-background px-2 py-0.5 font-tech text-[9px] disabled:opacity-50";

function fmtVal(v: unknown): string {
  if (v === undefined) return "—";
  return typeof v === "object" ? JSON.stringify(v) : String(v);
}

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

/** Editor de merge campo a campo (DESKTOP-33): dominante elegible + fuente
 * (A/B) por campo, prellenado desde el dominante, con preview del resultado. */
function MergeEditor({
  p,
  dominant,
  sources,
  onDominant,
  onSource,
  onSave,
  onCancel,
  busy,
}: {
  p: CandidatePair;
  dominant: FieldSource;
  sources: Record<string, FieldSource>;
  onDominant: (d: FieldSource) => void;
  onSource: (field: string, d: FieldSource) => void;
  onSave: () => void;
  onCancel: () => void;
  busy: boolean;
}) {
  const metaKeys = [
    ...new Set([...Object.keys(p.a.metadata ?? {}), ...Object.keys(p.b.metadata ?? {})]),
  ];
  const merged = mergeFields(p.a, p.b, sources);
  const fields: { key: string; va: string; vb: string }[] = [
    { key: "text", va: p.a.text, vb: p.b.text },
    ...metaKeys.map((k) => ({
      key: k,
      va: fmtVal(p.a.metadata?.[k]),
      vb: fmtVal(p.b.metadata?.[k]),
    })),
  ];
  return (
    <div className="space-y-2 border-t-2 border-foreground p-2">
      <div className="flex flex-wrap items-center gap-2 font-tech text-[9px] uppercase tracking-widest">
        <span className="text-muted-foreground">dominante (id vigente):</span>
        {(["a", "b"] as const).map((d) => (
          <label
            key={d}
            className={`press cursor-pointer border-2 px-2 py-0.5 ${
              dominant === d ? "border-neon bg-neon font-bold text-background" : "border-foreground bg-background"
            }`}
          >
            <input
              type="radio"
              name={`dom-${pairKey(p.a.id, p.b.id)}`}
              checked={dominant === d}
              onChange={() => onDominant(d)}
              className="sr-only"
            />
            {d === "a" ? p.a.id : p.b.id}
          </label>
        ))}
      </div>
      {fields.map((f) => (
        <div key={f.key} className="flex items-start gap-2">
          <span className="w-24 shrink-0 truncate pt-0.5 font-tech text-[9px] uppercase tracking-widest text-muted-foreground">
            {f.key}
          </span>
          <div className="flex shrink-0 gap-1">
            {(["a", "b"] as const).map((d) => (
              <button
                key={d}
                type="button"
                onClick={() => onSource(f.key, d)}
                disabled={busy}
                title={`${d}: ${(d === "a" ? f.va : f.vb).slice(0, 120)}`}
                className={`press border-2 px-1.5 font-tech text-[9px] disabled:opacity-50 ${
                  sources[f.key] === d
                    ? "border-neon bg-neon font-bold text-background"
                    : "border-foreground bg-background"
                }`}
              >
                {d.toUpperCase()}
              </button>
            ))}
          </div>
          <span className="min-w-0 flex-1 truncate pt-0.5 text-xs">{sources[f.key] === "b" ? f.vb : f.va}</span>
        </div>
      ))}
      <div className="border-2 border-dashed border-foreground bg-background p-2 text-xs leading-relaxed whitespace-pre-wrap break-words">
        {merged.text}
      </div>
      <div className="flex gap-2">
        <button
          type="button"
          onClick={onSave}
          disabled={busy}
          className="press border-2 border-foreground bg-neon px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-widest text-background disabled:opacity-50"
        >
          <ShieldCheck className="mr-1 inline h-3.5 w-3.5 align-[-2px]" strokeWidth={2.5} aria-hidden="true" />
          Guardar merge
        </button>
        <button type="button" onClick={onCancel} disabled={busy} className={BTN}>
          cancelar
        </button>
      </div>
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
  // DESKTOP-33: editor activo + selección batch (keys canónicas de par) +
  // confirmación destructiva pendiente + progress textual de operaciones.
  const [editing, setEditing] = useState<{
    p: CandidatePair;
    dominant: FieldSource;
    sources: Record<string, FieldSource>;
  } | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [confirming, setConfirming] = useState<PairRecord[] | null>(null);
  const [progress, setProgress] = useState("");

  const busy = detecting || progress !== "";
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

      // 2. MEM-58: sobre Tauri, el run REAL es el context engine embebido
      // (`assemble_with_recall` vía vanta_context_assemble): el outcome es el
      // report del engine (modo/tokens). Cualquier fallo (conexión no native,
      // comando ausente) cae al fallback heurístico D16a de abajo, intacto.
      if (transport instanceof TauriBackend) {
        try {
          const out = await contextAssemble({
            messages: toHistory(recs),
            budget_tokens: ASSEMBLE_BUDGET_TOKENS,
          });
          setRecords(recs);
          setPairs(null);
          const summary = formatAssembleReport(out);
          setRunInfo(`engine real · ${summary}`);
          onNotice(`Consolidación (engine real): ${summary}`);
          return;
        } catch {
          // Sin pipeline real disponible → fallback heurístico.
        }
      }

      // 2b. Detección (fallback D16a): search kNN con el texto de cada registro.
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
      setSelected(new Set());
      setEditing(null);
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

  /** Snapshot completo vía get() antes del soft-delete → Ctrl+Z restaura
   * también vector/ttl (undoStore.softDelete, DESKTOP-30). */
  async function discardToTrash(rec: PairRecord): Promise<void> {
    const full = await get(rec.id, rec.namespace);
    await undoStore.softDelete(full);
  }

  /** Merge guardado: put del registro merged sobre el dominante + papelera del
   * supersedido (undo disponible). La lista se actualiza reactivamente. */
  async function saveMerge() {
    if (!editing) return;
    const { p, dominant, sources } = editing;
    const kept = dominant === "a" ? p.a : p.b;
    const sup = dominant === "a" ? p.b : p.a;
    const merged = mergeFields(p.a, p.b, sources);
    setProgress(`mergeando…`);
    try {
      await vantaPut({
        namespace: kept.namespace,
        key: kept.id,
        payload: merged.text,
        metadata: merged.metadata,
      });
      await discardToTrash(sup);
      setRecords((prev) =>
        prev
          ? prev
              .filter((r) => r.id !== sup.id)
              .map((r) => (r.id === kept.id ? { ...r, text: merged.text, metadata: merged.metadata } : r))
          : prev,
      );
      setPairs((prev) => (prev ? prev.filter((x) => x.a.id !== sup.id && x.b.id !== sup.id) : prev));
      setEditing(null);
      onNotice(`merge guardado en ${kept.namespace}/${kept.id} · ${sup.id} movido a papelera (Ctrl+Z restaura)`);
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setProgress("");
    }
  }

  /** Confirmación ejecutada: papelera (soft-delete + undo) o remove duro. */
  async function handleConfirm(mode: "trash" | "purge") {
    if (!confirming) return;
    const targets = confirming;
    try {
      for (let i = 0; i < targets.length; i++) {
        const t = targets[i];
        setProgress(`eliminando ${i + 1}/${targets.length}…`);
        if (mode === "trash") {
          await discardToTrash(t);
        } else {
          await remove(t.id, t.namespace);
        }
      }
      const ids = new Set(targets.map((t) => t.id));
      setRecords((prev) => (prev ? prev.filter((r) => !ids.has(r.id)) : prev));
      setPairs((prev) => (prev ? prev.filter((x) => !ids.has(x.a.id) && !ids.has(x.b.id)) : prev));
      setSelected(new Set());
      setConfirming(null);
      onNotice(
        mode === "trash"
          ? `${targets.length} registro(s) movido(s) a papelera (Ctrl+Z restaura)`
          : `${targets.length} registro(s) eliminado(s) definitivamente`,
      );
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setProgress("");
    }
  }

  function toggleSelect(key: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  /** Batch marca dirección sobre todos los pares seleccionados. */
  async function batchMark(loser: "a" | "b") {
    if (!pairs) return;
    const chosen = pairs.filter((p) => selected.has(pairKey(p.a.id, p.b.id)));
    for (let i = 0; i < chosen.length; i++) {
      const p = chosen[i];
      setProgress(`marcando ${i + 1}/${chosen.length}…`);
      await markSuperseded(loser === "a" ? p.a : p.b, loser === "a" ? p.b : p.a);
    }
    setProgress("");
  }

  /** Batch descarta los miembros ya marcados como superados dentro de los pares
   * seleccionados (siempre vía confirmación 2 pasos). */
  function batchDiscardMarked() {
    if (!pairs) return;
    const targets = pairs
      .filter((p) => selected.has(pairKey(p.a.id, p.b.id)))
      .flatMap((p) => [p.a, p.b])
      .filter((r) => supersededBy(r.metadata) !== null);
    if (targets.length === 0) {
      onNotice("sin registros marcados como superados en la selección — marcá dirección primero");
      return;
    }
    setConfirming(targets);
  }

  /** Miembros ya marcados como superados dentro de un par (delete individual). */
  function markedInPair(p: CandidatePair): PairRecord[] {
    return [p.a, p.b].filter((r) => supersededBy(r.metadata) !== null);
  }

  const allSelected = !!pairs && pairs.length > 0 && selected.size === pairs.length;

  return (
    <section aria-label="Consolidación asistida" className="space-y-4">
      {/* Header (UX-01: LensShell compartido) */}
      <LensShell title="CONSOLIDAR" icon="⇄" meta="duplicados · superados · diff" />

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
          disabled={busy}
          className="press border-2 border-foreground bg-background px-3 py-1 text-xs font-semibold disabled:opacity-50"
        >
          {detecting ? "detectando…" : "⛃ Detectar candidatos"}
        </button>
        {supersededCount > 0 && (
          <span className="font-tech text-[10px] uppercase tracking-widest text-neon">
            {supersededCount} marcado(s)
          </span>
        )}
        {progress && (
          <span className="font-tech text-[10px] uppercase tracking-widest text-neon">{progress}</span>
        )}
        {runInfo && <span className="ml-auto font-tech text-[9px] text-muted-foreground">{runInfo}</span>}
      </div>

      {/* Toolbar batch */}
      {pairs !== null && pairs.length > 0 && (
        <div className="flex flex-wrap items-center gap-2 border-2 border-dashed border-foreground bg-muted px-2 py-1">
          <label className="flex cursor-pointer items-center gap-1 font-tech text-[9px] uppercase tracking-widest">
            <input type="checkbox" checked={!!allSelected} onChange={() => setSelected(allSelected ? new Set() : new Set(pairs.map((p) => pairKey(p.a.id, p.b.id))))} />
            todos
          </label>
          <span className="font-tech text-[9px] uppercase tracking-widest text-muted-foreground">
            {selected.size} seleccionado(s)
          </span>
          <button type="button" onClick={() => batchMark("a")} disabled={busy || selected.size === 0} className={BTN} title="Marca A como superado por B en cada par seleccionado">
            superar a→b ({selected.size})
          </button>
          <button type="button" onClick={() => batchMark("b")} disabled={busy || selected.size === 0} className={BTN} title="Marca B como superado por A en cada par seleccionado">
            superar b→a ({selected.size})
          </button>
          <button
            type="button"
            onClick={batchDiscardMarked}
            disabled={busy || selected.size === 0}
            className={BTN}
            title="Mueve los miembros superados de los pares seleccionados (con confirmación)"
          >
            ✕ descartar superados…
          </button>
        </div>
      )}

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
      {pairs !== null &&
        pairs.length > 0 &&
        pairs.map((p) => {
          const key = pairKey(p.a.id, p.b.id);
          const checked = selected.has(key);
          const marked = markedInPair(p);
          return (
            <article key={key} className="border-2 border-foreground bg-card">
              <div className="flex flex-wrap items-center justify-between gap-2 border-b-2 border-foreground bg-muted px-2 py-1">
                <div className="flex min-w-0 items-center gap-2">
                  <input
                    type="checkbox"
                    aria-label={`Seleccionar par ${key}`}
                    checked={checked}
                    onChange={() => toggleSelect(key)}
                  />
                  <span className="font-tech text-[9px] uppercase tracking-widest text-neon">
                    par · score {p.score.toFixed(4)} · similitud {fmtSim(p.score, maxScore).label}
                  </span>
                </div>
                <div className="flex flex-wrap gap-1">
                  <button
                    type="button"
                    onClick={() =>
                      setEditing({
                        p,
                        dominant: "a",
                        sources: defaultSources(p.a, p.b, "a"),
                      })
                    }
                    disabled={busy}
                    className={BTN}
                    title="Revisar lado a lado y mergear campo a campo"
                  >
                    <ShieldCheck className="mr-1 inline h-3.5 w-3.5 align-[-2px]" strokeWidth={2.5} aria-hidden="true" />
                    revisar/mergear
                  </button>
                  {marked.length > 0 && (
                    <button
                      type="button"
                      onClick={() => setConfirming(marked)}
                      disabled={busy}
                      className={BTN}
                      title={`Descartar ${marked.map((m) => m.id).join(", ")}`}
                    >
                      ✕ descartar superado{marked.length > 1 ? "s" : ""}…
                    </button>
                  )}
                </div>
              </div>
              {editing?.p === p ? (
                <MergeEditor
                  p={p}
                  dominant={editing.dominant}
                  sources={editing.sources}
                  busy={progress !== ""}
                  onDominant={(d) =>
                    setEditing({ p, dominant: d, sources: defaultSources(p.a, p.b, d) })
                  }
                  onSource={(field, d) =>
                    setEditing((prev) =>
                      prev && prev.p === p ? { ...prev, sources: { ...prev.sources, [field]: d } } : prev,
                    )
                  }
                  onSave={saveMerge}
                  onCancel={() => setEditing(null)}
                />
              ) : (
                <>
                  <div className="flex flex-col gap-2 p-2 sm:flex-row">
                    <RecordCard rec={p.a} score={p.score} maxScore={maxScore} />
                    <RecordCard rec={p.b} score={p.score} maxScore={maxScore} />
                  </div>
                  <div className="flex flex-wrap gap-1 border-t-2 border-foreground px-2 py-1">
                    <button
                      type="button"
                      onClick={() => markSuperseded(p.a, p.b)}
                      disabled={busy}
                      className={BTN}
                      title={`Escribe metadata.superseded_by = ${p.b.id} en ${p.a.id}`}
                    >
                      → {p.a.id} superado por {p.b.id}
                    </button>
                    <button
                      type="button"
                      onClick={() => markSuperseded(p.b, p.a)}
                      disabled={busy}
                      className={BTN}
                      title={`Escribe metadata.superseded_by = ${p.a.id} en ${p.b.id}`}
                    >
                      → {p.b.id} superado por {p.a.id}
                    </button>
                  </div>
                </>
              )}
            </article>
          );
        })}

      {/* Confirmación destructiva (2 pasos) */}
      {confirming !== null && (
        <ConfirmDiscard
          targets={confirming}
          busy={progress !== ""}
          onClose={() => setConfirming(null)}
          onConfirm={handleConfirm}
        />
      )}
    </section>
  );
}

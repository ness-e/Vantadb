// Historial tab (VS-14): lista de versiones retenidas (v1..vN con timestamp
// vía `versions`), diff entre dos versiones seleccionadas — payload (line-diff),
// metadata (KV añadido/quitado/cambiado), vector (dimensión/norma + "cambió")
// vía `getVersion` — y REVERTIR a vN como acción explícita (botón →
// confirmación → vantaPut; P6, nunca implícito). Encoding redundante VS-18:
// ningún estado depende solo del color (ícono + texto siempre acompañan).
import { useCallback, useEffect, useState } from "react";
import type { MemoryRecord } from "../../vanta";
import { getVersion, vantaErrorMessage, vantaPut, versions } from "../../vanta";
import { diffVersions, type VersionDiff } from "./historial-diff";
import { fmtDateTime, fmtRelative } from "./shared";
import { TriangleAlert } from "lucide-react";
import { tp, tt, type DesktopLang } from "../../i18n";
import { connectionPrefs } from "../../store/connections";

interface Props {
  record: MemoryRecord;
  /** Record actualizado tras `vantaPut` (revert crea versión nueva). */
  onSaved: (updated: MemoryRecord) => void;
  onError: (msg: string) => void;
  lang?: DesktopLang;
}

/** Techo de líneas renderizadas en el diff de payload (evita scroll infinito). */
const MAX_DIFF_LINES = 60;

function fmtValue(v: unknown): string {
  if (v === null) return "null";
  if (typeof v === "string") return v;
  if (typeof v === "object") return JSON.stringify(v);
  return String(v);
}

export default function HistorialTab({ record, onSaved, onError, lang = connectionPrefs.get().lang ?? "es" }: Props) {
  const [vers, setVers] = useState<MemoryRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [listError, setListError] = useState<string | null>(null);
  const [baseVer, setBaseVer] = useState<number | null>(null);
  const [cmpVer, setCmpVer] = useState<number | null>(null);
  const [diff, setDiff] = useState<VersionDiff | null>(null);
  const [diffLoading, setDiffLoading] = useState(false);
  const [diffError, setDiffError] = useState<string | null>(null);
  const [confirmVer, setConfirmVer] = useState<number | null>(null);
  const [reverting, setReverting] = useState(false);
  const [revertFlash, setRevertFlash] = useState<number | null>(null);

  // Al cambiar de registro (id/ns) se descartan selecciones y confirmaciones.
  useEffect(() => {
    setBaseVer(null);
    setCmpVer(null);
    setDiff(null);
    setDiffError(null);
    setConfirmVer(null);
  }, [record.id, record.namespace]);

  const loadVersions = useCallback(async () => {
    setLoading(true);
    setListError(null);
    try {
      const list = await versions(record.id, record.namespace);
      setVers(list);
      // Default del diff: las dos versiones más recientes (vN-1 ▸ vN).
      setBaseVer((cur) => cur ?? (list.length >= 2 ? (list[list.length - 2].version ?? null) : null));
      setCmpVer((cur) => cur ?? (list.length >= 1 ? (list[list.length - 1].version ?? null) : null));
    } catch (err) {
      setListError(vantaErrorMessage(err));
    } finally {
      setLoading(false);
    }
  }, [record.id, record.namespace]);

  useEffect(() => {
    void loadVersions();
  }, [loadVersions]);

  // Diff de las dos versiones seleccionadas — point-reads autoritativos
  // (getVersion), no reusa el listado por si la lista se recortó/evictó.
  useEffect(() => {
    if (baseVer == null || cmpVer == null) {
      setDiff(null);
      return;
    }
    let alive = true;
    setDiffLoading(true);
    setDiffError(null);
    (async () => {
      try {
        const [a, b] = await Promise.all([
          getVersion(record.id, baseVer, record.namespace),
          getVersion(record.id, cmpVer, record.namespace),
        ]);
        if (!alive) return;
        setDiff(diffVersions(a, b));
      } catch (err) {
        if (alive) setDiffError(vantaErrorMessage(err));
      } finally {
        if (alive) setDiffLoading(false);
      }
    })();
    return () => {
      alive = false;
    };
  }, [baseVer, cmpVer, record.id, record.namespace]);

  const confirmTarget =
    confirmVer != null ? (vers.find((v) => v.version === confirmVer) ?? null) : null;

  async function handleRevert() {
    if (!confirmTarget) return;
    setReverting(true);
    try {
      const updated = await vantaPut({
        namespace: record.namespace,
        key: record.id,
        payload: confirmTarget.text,
        metadata: confirmTarget.metadata,
        expires_at_ms: confirmTarget.expires_at_ms ?? undefined,
      });
      onSaved(updated);
      setConfirmVer(null);
      setRevertFlash(confirmTarget.version ?? 0);
      window.setTimeout(() => setRevertFlash(null), 2500);
      // La lista cambió (revert creó vN+1); refrescar sin esperar al shell.
      await loadVersions();
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setReverting(false);
    }
  }

  const hasVersions = vers.length > 0;

  return (
    <div>
      {revertFlash != null && (
        <p className="mb-2 border-2 border-foreground bg-neon px-2 py-1 font-tech text-[10px] font-bold text-background">
          {tp(lang, "inspector.histReverted", "✓ revertido a v{v} (versión nueva creada)", { v: String(revertFlash) })}
        </p>
      )}

      {/* Lista de versiones */}
      <div className="border-2 border-foreground bg-background p-2">
        <div className="flex items-center justify-between">
          <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
            <span aria-hidden="true">≡</span> {tp(lang, "inspector.histVersions", "versiones ({n})", { n: String(vers.length) })}
          </span>
          {loading && (
            <span className="font-tech text-[9px] uppercase text-neon">{tt(lang, "inspector.histLoading", "cargando…")}</span>
          )}
        </div>

        {listError ? (
          <p className="mt-1 font-tech text-[10px] text-destructive">✕ {listError}</p>
        ) : vers.length === 0 && !loading ? (
          <p className="mt-1 font-tech text-[10px] text-muted-foreground">
            {tt(lang, "inspector.histEmpty", "sin versiones retenidas — el bridge nativo retiene hasta 32 por registro")}
          </p>
        ) : (
          <div className="mt-1 space-y-1">
            {vers.map((v) => {
              const ver = v.version ?? 0;
              const isActual = ver === record.version;
              return (
                <div
                  key={ver}
                  className={`flex items-center gap-2 border-2 border-foreground px-2 py-1 ${
                    isActual ? "bg-neon text-background" : "bg-card"
                  }`}
                >
                  <span className="shrink-0 font-tech text-[10px] font-bold">v{ver}</span>
                  <span
                    className={`min-w-0 flex-1 truncate font-tech text-[10px] ${
                      isActual ? "text-background" : "text-muted-foreground"
                    }`}
                    title={v.updated_at_ms != null ? fmtDateTime(v.updated_at_ms) : undefined}
                  >
                    {v.updated_at_ms != null
                      ? `${fmtDateTime(v.updated_at_ms)} (${fmtRelative(v.updated_at_ms, Date.now())})`
                      : "—"}
                  </span>
                  {isActual ? (
                    <span className="shrink-0 font-tech text-[10px] font-bold">
                      <span aria-hidden="true">●</span> {tt(lang, "inspector.histActual", "ACTUAL")}
                    </span>
                  ) : (
                    <button
                      type="button"
                      onClick={() => setConfirmVer(ver)}
                      className="press shrink-0 border-2 border-foreground bg-background px-1.5 py-0.5 font-tech text-[9px] font-bold"
                      title={tp(lang, "inspector.histRevertTitle", "Revertir el registro al contenido de v{v} (crea una versión nueva)", { v: String(ver) })}
                    >
                      <span aria-hidden="true">↺</span> {tt(lang, "inspector.histRevert", "REVERTIR")}
                    </button>
                  )}
                </div>
              );
            })}
          </div>
        )}

        {hasVersions && vers.length === 1 && (
          <p className="mt-1 font-tech text-[10px] text-muted-foreground">
            {tt(lang, "inspector.histSingle", "1 sola versión — editá el registro y guardá para crear v2")}
          </p>
        )}
      </div>

      {/* Selección de diff */}
      {hasVersions && vers.length >= 2 && (
        <div className="mt-2 flex items-center gap-2 border-2 border-foreground bg-background p-2">
          <label className="flex items-center gap-1 font-tech text-[9px] uppercase tracking-widest text-muted-foreground">
            {tt(lang, "inspector.histFrom", "desde")}
            <select
              value={baseVer ?? ""}
              onChange={(e) => setBaseVer(e.target.value === "" ? null : Number(e.target.value))}
              className="border-2 border-foreground bg-card px-1 py-0.5 font-tech text-[10px]"
            >
              {vers.map((v) => (
                <option key={v.version} value={v.version ?? ""}>
                  v{v.version}
                </option>
              ))}
            </select>
          </label>
          <span aria-hidden="true" className="text-neon">
            ▸
          </span>
          <label className="flex items-center gap-1 font-tech text-[9px] uppercase tracking-widest text-muted-foreground">
            {tt(lang, "inspector.ttlAbsolute", "hasta")}
            <select
              value={cmpVer ?? ""}
              onChange={(e) => setCmpVer(e.target.value === "" ? null : Number(e.target.value))}
              className="border-2 border-foreground bg-card px-1 py-0.5 font-tech text-[10px]"
            >
              {vers.map((v) => (
                <option key={v.version} value={v.version ?? ""}>
                  v{v.version}
                </option>
              ))}
            </select>
          </label>
        </div>
      )}

      {/* Confirmación de revert — P6: acción explícita, nunca implícita */}
      {confirmTarget && (
        <div className="mt-2 border-2 border-foreground bg-background p-2">
          <p className="font-tech text-[10px] font-bold uppercase tracking-widest text-neon">
            <span aria-hidden="true">
              <TriangleAlert className="mr-1 inline h-3 w-3 align-[-2px]" strokeWidth={2.5} />
            </span>{" "}
            {tp(lang, "inspector.histConfirmQ", "¿revertir a v{v}?", { v: String(confirmTarget.version) })}
          </p>
          <p className="mt-1 font-tech text-[10px] text-muted-foreground">
            {tp(lang, "inspector.histConfirmDesc", "restaura payload + metadata + TTL de v{v}. Crea una versión nueva (v{next}) — el vector no se restaura (vantaPut no lo acepta en Fase 0).", { v: String(confirmTarget.version), next: String((record.version ?? 0) + 1) })}
          </p>
          <div className="mt-2 flex gap-2">
            <button
              type="button"
              onClick={() => setConfirmVer(null)}
              disabled={reverting}
              className="press flex-1 border-2 border-foreground bg-card px-2 py-1 font-tech text-[10px]"
            >
              {tt(lang, "inspector.histCancel", "CANCELAR")}
            </button>
            <button
              type="button"
              onClick={handleRevert}
              disabled={reverting}
              className="press flex-1 border-2 border-foreground bg-foreground px-2 py-1 font-tech text-[10px] font-bold text-background"
            >
              {reverting ? tt(lang, "inspector.histReverting", "REVIRTIENDO…") : tt(lang, "inspector.histConfirm", "CONFIRMAR")}
            </button>
          </div>
        </div>
      )}

      {/* Diff */}
      {baseVer != null && cmpVer != null && (
        <div className="mt-3">
          {diffLoading ? (
            <p className="font-tech text-[10px] uppercase text-muted-foreground">
              {tt(lang, "inspector.histCalcDiff", "calculando diff…")}
            </p>
          ) : diffError ? (
            <p className="border-2 border-foreground bg-background p-2 font-tech text-[10px] text-neon">
              ✕ {diffError} {tt(lang, "inspector.histEvicted", "— puede que la versión haya sido evictada (cap 32); elegí otra.")}
            </p>
          ) : diff ? (
            <div className="space-y-3">
              <div className="flex items-center justify-between border-2 border-foreground bg-background px-2 py-1">
                <span className="font-tech text-[10px] font-bold">
                  <span aria-hidden="true">◆</span> v{baseVer} <span className="text-neon">▸</span> v{cmpVer}
                </span>
                <span className="font-tech text-[9px] uppercase tracking-widest text-muted-foreground">
                  diff
                </span>
              </div>

              {/* Payload: line-diff (encoding redundante: +/− + color + texto) */}
              <div className="border-2 border-foreground bg-background p-2">
                <div className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                  <span aria-hidden="true">≡</span> payload
                </div>
                {diff.payload.length === 0 ? (
                  <p className="mt-1 font-tech text-[10px] text-muted-foreground">
                    {tt(lang, "inspector.histEmptyBoth", "vacío en ambas versiones")}
                  </p>
                ) : diff.payload.every((l) => l.kind === "ctx") ? (
                  <p className="mt-1 font-tech text-[10px] text-muted-foreground">
                    <span aria-hidden="true">✓</span> {tt(lang, "inspector.histNoChanges", "sin cambios")}
                  </p>
                ) : (
                  <>
                    <div className="mt-1 max-h-40 overflow-y-auto font-mono text-[10px] leading-relaxed">
                      {diff.payload.slice(0, MAX_DIFF_LINES).map((l, i) => (
                        <div
                          key={i}
                          className={`whitespace-pre-wrap break-words ${
                            l.kind === "add"
                              ? "text-neon"
                              : l.kind === "del"
                                ? "text-muted-foreground line-through"
                                : "text-foreground"
                          }`}
                        >
                          <span aria-hidden="true">
                            {l.kind === "add" ? "+ " : l.kind === "del" ? "− " : "  "}
                          </span>
                          {l.text || " "}
                        </div>
                      ))}
                      {diff.payload.length > MAX_DIFF_LINES && (
                        <p className="font-tech text-[9px] text-muted-foreground">
                          {tp(lang, "inspector.histHiddenLines", "… +{n} líneas ocultas", { n: String(diff.payload.length - MAX_DIFF_LINES) })}
                        </p>
                      )}
                    </div>
                    <p className="mt-1 font-tech text-[9px] text-muted-foreground">
                      <span aria-hidden="true">+</span>{" "}
                      {tp(lang, "inspector.histAddedCount", "{a} añadidas", { a: String(diff.payload.filter((l) => l.kind === "add").length) })} ·{" "}
                      <span aria-hidden="true">−</span>{" "}
                      {tp(lang, "inspector.histRemovedCount", "{d} quitadas", { d: String(diff.payload.filter((l) => l.kind === "del").length) })}
                    </p>
                  </>
                )}
              </div>

              {/* Metadata: KV diff añadido/quitado/cambiado */}
              <div className="border-2 border-foreground bg-background p-2">
                <div className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                  <span aria-hidden="true">▦</span> metadata
                </div>
                {diff.metadata.added.length === 0 &&
                diff.metadata.removed.length === 0 &&
                diff.metadata.changed.length === 0 ? (
                  <p className="mt-1 font-tech text-[10px] text-muted-foreground">
                    <span aria-hidden="true">✓</span> {tt(lang, "inspector.histNoChanges", "sin cambios")}
                  </p>
                ) : (
                  <div className="mt-1 space-y-1 font-tech text-[10px]">
                    {diff.metadata.added.length > 0 && (
                      <p className="break-all">
                        <span className="font-bold text-neon">{tt(lang, "inspector.histAddedLabel", "+ añadido:")}</span>{" "}
                        {diff.metadata.added.join(", ")}
                      </p>
                    )}
                    {diff.metadata.removed.length > 0 && (
                      <p className="break-all">
                        <span className="font-bold text-muted-foreground">{tt(lang, "inspector.histRemovedLabel", "− quitado:")}</span>{" "}
                        {diff.metadata.removed.join(", ")}
                      </p>
                    )}
                    {diff.metadata.changed.map((c) => (
                      <p key={c.key} className="break-all">
                        <span className="font-bold">~ {c.key}:</span>{" "}
                        <span className="text-muted-foreground line-through">
                          {fmtValue(c.before)}
                        </span>{" "}
                        <span className="text-neon">→ {fmtValue(c.after)}</span>
                      </p>
                    ))}
                  </div>
                )}
              </div>

              {/* Vector: norma/dim + "cambió" sí/no */}
              <div className="border-2 border-foreground bg-background p-2">
                <div className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                  <span aria-hidden="true">▤</span> vector
                </div>
                <div className="mt-1 space-y-1 font-tech text-[10px]">
                  {diff.vecA && diff.vecB ? (
                    <>
                      <p className={diff.vectorChanged ? "font-bold text-neon" : "text-muted-foreground"}>
                        <span aria-hidden="true">{diff.vectorChanged ? "✕" : "✓"}</span> {tp(lang, "inspector.histVecChanged", "vector cambió: {v}", { v: diff.vectorChanged ? tt(lang, "inspector.histVecYes", "SÍ") : tt(lang, "inspector.histVecNo", "no") })}
                      </p>
                      <p className="text-muted-foreground">
                        v{baseVer}: {diff.vecA.dim}d · norma {diff.vecA.norm.toFixed(4)} → v{cmpVer}:{" "}
                        {diff.vecB.dim}d · norma {diff.vecB.norm.toFixed(4)}
                      </p>
                    </>
                  ) : diff.vecA || diff.vecB ? (
                    <p className="font-bold text-neon">
                      <span aria-hidden="true">✕</span> {tp(lang, "inspector.histVecChanged", "vector cambió: {v}", { v: tp(lang, "inspector.histVecOnlyHere", "solo en v{v} ({d}d)", { v: String(diff.vecA ? baseVer : cmpVer), d: String((diff.vecA ?? diff.vecB)?.dim) }) })}
                    </p>
                  ) : (
                    <p className="text-muted-foreground">
                      <span aria-hidden="true">−</span> {tt(lang, "inspector.histNoVector", "sin vector en ninguna")}
                    </p>
                  )}
                </div>
              </div>
            </div>
          ) : null}
        </div>
      )}
    </div>
  );
}
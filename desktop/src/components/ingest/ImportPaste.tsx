// OP-01 — Modal de import CSV/JSON pegado. Patrón manga/linocut de
// CommandPalette/ActivityPanel: overlay + panel press-lg border-4, títulos
// font-display, labels font-tech, acentos text-neon. Flujo: pegar → preview
// auto-parseado (filas ✓/✗) → IMPORTAR → runImport(ingestBatch) → reporte.
// Los errores nunca son silenciosos: parse global → alert; filas inválidas →
// marcadas en preview y repetidas en el reporte.
import { useEffect, useMemo, useRef, useState } from "react";
import { ingestBatch, vantaErrorMessage } from "../../vanta";
import {
  EXAMPLE_CSV,
  MAX_IMPORT,
  parseImport,
  runImport,
  type ImportReport,
  type ParseResult,
} from "./parseImport";

interface Props {
  open: boolean;
  onClose: () => void;
  defaultNamespace: string;
  /** N registros importados con éxito (el shell refresca el grid). */
  onImported: (count: number) => void;
  /** Error inesperado fuera del reporte (fallo del bridge no capturado). */
  onError: (msg: string) => void;
}

const PREVIEW_ROWS = 8;

function textSnippet(item: { text: string } | null, error?: string): string {
  if (!item) return error ?? "—";
  return item.text.length > 64 ? `${item.text.slice(0, 64)}…` : item.text;
}

function metaSnippet(meta: Record<string, unknown> | undefined): string {
  if (!meta) return "—";
  const s = JSON.stringify(meta);
  return s.length > 40 ? `${s.slice(0, 40)}…` : s;
}

export default function ImportPaste({
  open,
  onClose,
  defaultNamespace,
  onImported,
  onError,
}: Props) {
  const [paste, setPaste] = useState("");
  const [ns, setNs] = useState(defaultNamespace);
  const [busy, setBusy] = useState(false);
  const [report, setReport] = useState<ImportReport | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Reset al abrir + autofocus.
  useEffect(() => {
    if (open) {
      setPaste("");
      setNs(defaultNamespace);
      setReport(null);
      setBusy(false);
      requestAnimationFrame(() => textareaRef.current?.focus());
    }
  }, [open, defaultNamespace]);

  // Escape cierra (salvo mientras importa).
  useEffect(() => {
    if (!open || busy) return;
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, busy, onClose]);

  // Auto-parse: el textarea ES el editor (re-editar → re-preview).
  const parsed: ParseResult | null = useMemo(
    () => (paste.trim() ? parseImport(paste, ns.trim() || defaultNamespace) : null),
    [paste, ns, defaultNamespace],
  );

  if (!open) return null;

  const targetNs = ns.trim() || defaultNamespace;
  const rows = parsed?.rows ?? [];
  const preview = rows.slice(0, PREVIEW_ROWS);

  async function handleImport() {
    if (!parsed || parsed.valid === 0) return;
    if (!window.confirm(`Importar ${parsed.valid} registros a ns "${targetNs}"?`)) return;
    setBusy(true);
    try {
      const r = await runImport(rows.filter((x) => x.item !== null), ingestBatch);
      setReport(r);
      if (r.imported > 0) onImported(r.imported);
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  const btnBase =
    "press border-2 border-foreground px-2 py-1 font-tech text-[10px] uppercase tracking-widest";
  const disabled = busy || !parsed || parsed.valid === 0;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto bg-black/55 p-6"
      role="dialog"
      aria-modal="true"
      aria-label="Importar CSV o JSON pegado"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget && !busy) onClose();
      }}
    >
      <section className="press-lg w-full max-w-2xl border-4 border-foreground bg-background">
        <header className="flex items-center gap-2 border-b-4 border-foreground bg-card px-4 py-3">
          <h2 className="font-display text-2xl text-stencil">IMPORT CSV/JSON</h2>
          <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
            máx {MAX_IMPORT} registros · ns: {targetNs}
          </span>
          <button
            className={`${btnBase} ml-auto bg-background`}
            onClick={onClose}
            disabled={busy}
            aria-label="Cerrar"
          >
            ✕ CERRAR
          </button>
        </header>

        {parsed?.error && (
          <p role="alert" className="border-b-4 border-foreground bg-card px-4 py-2 font-tech text-[11px] text-neon">
            ⚠ {parsed.error}
          </p>
        )}

        {report && (
          <div className="border-b-4 border-foreground bg-card px-4 py-3" aria-live="polite">
            <p className="font-tech text-[12px] font-bold uppercase tracking-widest text-neon">
              ✓ {report.imported} importados
            </p>
            {report.errors.length > 0 && (
              <ul className="mt-1 space-y-0.5">
                {report.errors.map((e, i) => (
                  <li key={i} className="font-tech text-[10px] text-muted-foreground">
                    filas {e.rows}: {e.message}
                  </li>
                ))}
              </ul>
            )}
            <button
              className={`${btnBase} mt-2 bg-background`}
              onClick={() => {
                setReport(null);
                setPaste("");
              }}
            >
              ↺ NUEVO PASTE
            </button>
          </div>
        )}

        <div className="space-y-3 p-4">
          <label className="block">
            <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              Pegá CSV (cabecera: key,payload,metadata_json) o JSON (array / NDJSON):
            </span>
            <textarea
              ref={textareaRef}
              value={paste}
              onChange={(e) => {
                setPaste(e.target.value);
                setReport(null);
              }}
              spellCheck={false}
              rows={7}
              placeholder="key,payload,metadata_json&#10;mi-key,texto a recordar,{&quot;tipo&quot;:&quot;nota&quot;}"
              className="mt-1 w-full resize-y border-2 border-foreground bg-background p-2 font-tech text-[12px] outline-none focus:border-neon"
            />
          </label>

          <div className="flex flex-wrap items-center gap-2">
            <label className="flex items-center gap-1 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              namespace
              <input
                value={ns}
                onChange={(e) => setNs(e.target.value)}
                className="w-40 border-2 border-foreground bg-background px-1.5 py-0.5 font-tech text-[11px] outline-none focus:border-neon"
              />
            </label>
            <button
              className={`${btnBase} bg-background`}
              onClick={() => setPaste(EXAMPLE_CSV)}
              disabled={busy}
              title="Cargar CSV de ejemplo"
            >
              ◇ EJEMPLO
            </button>
            <span className="ml-auto font-tech text-[10px] uppercase tracking-widest">
              {parsed ? (
                <>
                  <span className="text-foreground">{parsed.valid} ✓</span>
                  <span className="text-muted-foreground">
                    {" "}
                    · {parsed.invalid} ✗ · {parsed.truncated ? `solo primeros ${MAX_IMPORT}` : `${rows.length} filas`}
                  </span>
                </>
              ) : (
                <span className="text-muted-foreground">sin contenido</span>
              )}
            </span>
          </div>

          {parsed?.truncated && (
            <p className="font-tech text-[10px] text-neon">
              ⚠ el paste supera {MAX_IMPORT} registros — se importan solo los primeros.
            </p>
          )}

          <div className="overflow-x-auto border-2 border-foreground">
            <table className="w-full border-collapse bg-background">
              <thead>
                <tr className="border-b-2 border-foreground font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                  <th className="px-2 py-1 text-left">#</th>
                  <th className="px-2 py-1 text-left">estado</th>
                  <th className="px-2 py-1 text-left">key</th>
                  <th className="px-2 py-1 text-left">text</th>
                  <th className="px-2 py-1 text-left">ns</th>
                  <th className="px-2 py-1 text-left">metadata</th>
                </tr>
              </thead>
              <tbody>
                {preview.map((r) => (
                  <tr
                    key={r.index}
                    className="border-b border-foreground/30 font-tech text-[11px]"
                  >
                    <td className="px-2 py-1 text-muted-foreground">{r.index}</td>
                    <td className="px-2 py-1">
                      {r.item ? (
                        <span className="text-neon" aria-label="válida">
                          ✓
                        </span>
                      ) : (
                        <span className="text-foreground" title={r.error} aria-label="inválida">
                          ✗
                        </span>
                      )}
                    </td>
                    <td className="max-w-[120px] truncate px-2 py-1">{r.item?.id ?? <span className="text-muted-foreground">auto</span>}</td>
                    <td className="max-w-[240px] truncate px-2 py-1">
                      {r.item ? textSnippet(r.item) : <span className="text-neon">{r.error}</span>}
                    </td>
                    <td className="max-w-[80px] truncate px-2 py-1 text-muted-foreground">
                      {r.item?.namespace ?? "—"}
                    </td>
                    <td className="max-w-[140px] truncate px-2 py-1 text-muted-foreground">
                      {r.item ? metaSnippet(r.item.metadata) : "—"}
                    </td>
                  </tr>
                ))}
                {preview.length === 0 && (
                  <tr>
                    <td colSpan={6} className="p-3 text-center font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                      {parsed ? "sin filas parseables" : "el preview aparece al pegar"}
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
          {rows.length > PREVIEW_ROWS && (
            <p className="font-tech text-[10px] text-muted-foreground">
              … y {rows.length - PREVIEW_ROWS} más (mostrando primeras {PREVIEW_ROWS})
            </p>
          )}
        </div>

        <footer className="flex items-center gap-2 border-t-4 border-foreground bg-card px-4 py-3">
          {busy && (
            <span className="font-tech text-[10px] uppercase tracking-widest text-neon" role="status">
              importando…
            </span>
          )}
          <button
            className={`${btnBase} ml-auto bg-background`}
            onClick={handleImport}
            disabled={disabled}
            title="Importar registros válidos en chunks de 50"
          >
            ⤓ IMPORTAR {parsed?.valid ?? 0}
          </button>
        </footer>
      </section>
    </div>
  );
}

// WASM-04 — Modal de import por ARCHIVO: drag&drop + `<input type="file">`
// accesible. Mismo patrón visual/funcional que ImportPaste (OP-01): el texto
// del archivo ES el mismo input del paste → `parseImportFile` (parseImport /
// parseVdbDump) → preview ✓/✗ → `runImport(ingestBatch)` → reporte. Errores
// NUNCA silenciosos: parse global → alert; filas inválidas → marcadas y
// repetidas en el reporte; fallo de lectura/import → onError.
import { useEffect, useMemo, useRef, useState } from "react";
import { ingestBatch, vantaErrorMessage } from "../../vanta";
import {
  MAX_IMPORT,
  parseImportFile,
  runImport,
  type ImportReport,
  type ParseResult,
} from "./parseImport";
import { FileText, TriangleAlert } from "lucide-react";
// UX-03: trap de foco del dialog (Tab cicla, Escape cierra, foco restaurado).
import { useModalFocus } from "./useModalFocus";

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
const ACCEPT = ".csv,.json,.jsonl,.vdbdump";

function textSnippet(item: { text: string } | null, error?: string): string {
  if (!item) return error ?? "—";
  return item.text.length > 64 ? `${item.text.slice(0, 64)}…` : item.text;
}

function metaSnippet(meta: Record<string, unknown> | undefined): string {
  if (!meta) return "—";
  const s = JSON.stringify(meta);
  return s.length > 40 ? `${s.slice(0, 40)}…` : s;
}

export default function ImportDrop({
  open,
  onClose,
  defaultNamespace,
  onImported,
  onError,
}: Props) {
  const [fileName, setFileName] = useState<string | null>(null);
  const [fileText, setFileText] = useState("");
  const [ns, setNs] = useState(defaultNamespace);
  const [dragOver, setDragOver] = useState(false);
  const [busy, setBusy] = useState(false);
  const [report, setReport] = useState<ImportReport | null>(null);
  // UX-03: overlay del dialog para el trap de foco.
  const dialogRef = useRef<HTMLDivElement>(null);

  // Reset al abrir.
  useEffect(() => {
    if (open) {
      setFileName(null);
      setFileText("");
      setNs(defaultNamespace);
      setDragOver(false);
      setReport(null);
      setBusy(false);
    }
  }, [open, defaultNamespace]);

  // UX-03: trap de foco + Escape + restauración del foco al cerrar. Reemplaza
  // el useEffect de Escape anterior (el hook escucha en captura y hace
  // stopPropagation antes que handlers globales).
  useModalFocus(dialogRef, open, onClose, busy);

  // Auto-parse: el contenido del archivo ES el paste (mismo parser OP-01);
  // re-drop/re-selección → re-preview.
  const parsed: ParseResult | null = useMemo(
    () =>
      fileName && fileText.trim()
        ? parseImportFile(fileName, fileText, ns.trim() || defaultNamespace)
        : null,
    [fileName, fileText, ns, defaultNamespace],
  );

  if (!open) return null;

  const targetNs = ns.trim() || defaultNamespace;
  const rows = parsed?.rows ?? [];
  const preview = rows.slice(0, PREVIEW_ROWS);

  async function handleFile(file: File) {
    setReport(null);
    setFileName(file.name);
    try {
      setFileText(await file.text());
    } catch (err) {
      // Lectura fallida → error visible, no silencioso.
      setFileName(null);
      onError(vantaErrorMessage(err));
    }
  }

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
      ref={dialogRef}
      className="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto bg-black/55 p-6"
      role="dialog"
      aria-modal="true"
      aria-label="Importar archivo CSV, JSON, JSONL o VDBDUMP"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget && !busy) onClose();
      }}
    >
      <section className="press-lg w-full max-w-2xl border-4 border-foreground bg-background">
        <header className="flex items-center gap-2 border-b-4 border-foreground bg-card px-4 py-3">
          <h2 className="font-display text-2xl text-stencil">IMPORT ARCHIVO</h2>
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
          <p role="alert" className="border-b-4 border-foreground bg-card px-4 py-2 font-tech text-[11px] text-accent-text">
            <TriangleAlert className="mr-1 inline h-3 w-3 align-[-1px]" strokeWidth={2.5} aria-hidden="true" />
            {parsed.error}
          </p>
        )}

        {report && (
          <div className="border-b-4 border-foreground bg-card px-4 py-3" aria-live="polite">
            <p className="font-tech text-[12px] font-bold uppercase tracking-widest text-accent-text">
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
                setFileName(null);
                setFileText("");
              }}
            >
              ↺ NUEVO ARCHIVO
            </button>
          </div>
        )}

        <div className="space-y-3 p-4">
          {/* Drop zone = label del input file: clic abre el selector (fallback
              accesible), drag&drop alimenta el mismo handleFile. */}
          <label
            htmlFor="vdb-file-input"
            onDragOver={(e) => {
              e.preventDefault();
              setDragOver(true);
            }}
            onDragLeave={() => setDragOver(false)}
            onDrop={(e) => {
              e.preventDefault();
              setDragOver(false);
              const f = e.dataTransfer?.files?.[0];
              if (f) void handleFile(f);
            }}
            className={`flex min-h-28 cursor-pointer flex-col items-center justify-center gap-1 border-2 border-dashed border-foreground p-4 text-center font-tech text-[11px] uppercase tracking-widest transition-colors ${
              dragOver ? "border-neon bg-neon/10 text-accent-text" : "text-muted-foreground hover:border-neon"
            }`}
          >
            <input
              id="vdb-file-input"
              type="file"
              accept={ACCEPT}
              className="sr-only"
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (f) void handleFile(f);
                e.target.value = ""; // permite re-elegir el mismo archivo
              }}
            />
            {fileName ? (
              <span className="text-foreground" aria-live="polite">
                <FileText className="mr-1 inline h-4 w-4 align-[-3px]" strokeWidth={2.5} aria-hidden="true" />
                {fileName}
              </span>
            ) : (
              <>
                <span className="text-foreground">⤓ ARRASTRÁ UN ARCHIVO</span>
                <span>.csv · .json · .jsonl · .vdbdump — o hacé clic para elegir</span>
              </>
            )}
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
                <span className="text-muted-foreground">sin archivo</span>
              )}
            </span>
          </div>

          {parsed?.truncated && (
            <p className="font-tech text-[10px] text-accent-text">
              <TriangleAlert className="mr-1 inline h-3 w-3 align-[-1px]" strokeWidth={2.5} aria-hidden="true" />
              el archivo supera {MAX_IMPORT} registros — se importan solo los primeros.
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
                      {r.item ? textSnippet(r.item) : <span className="text-accent-text">{r.error}</span>}
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
                      {parsed ? "sin filas parseables" : "el preview aparece al elegir un archivo"}
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
            <span className="font-tech text-[10px] uppercase tracking-widest text-accent-text" role="status">
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
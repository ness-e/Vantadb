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
import { tp, tt, type DesktopLang } from "../../i18n";
import { connectionPrefs } from "../../store/connections";

interface Props {
  open: boolean;
  onClose: () => void;
  defaultNamespace: string;
  /** N registros importados con éxito (el shell refresca el grid). */
  onImported: (count: number) => void;
  /** Error inesperado fuera del reporte (fallo del bridge no capturado). */
  onError: (msg: string) => void;
  lang?: DesktopLang;
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
  lang = connectionPrefs.get().lang ?? "es",
}: Props) {
  const [fileName, setFileName] = useState<string | null>(null);
  const [fileText, setFileText] = useState("");
  const [ns, setNs] = useState(defaultNamespace);
  const [dragOver, setDragOver] = useState(false);
  const [busy, setBusy] = useState(false);
  const [report, setReport] = useState<ImportReport | null>(null);
  // UX-A11Y-01: confirmación inline (patrón IngestForm confirming) — el
  // window.confirm nativo rompía teclado/SR y el lenguaje de la app.
  const [confirming, setConfirming] = useState(false);
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
      setConfirming(false);
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
    setConfirming(false);
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
    if (!parsed || parsed.valid === 0 || busy) return;
    setConfirming(true);
  }

  async function doImport() {
    if (!parsed || parsed.valid === 0) return;
    setBusy(true);
    try {
      const r = await runImport(rows.filter((x) => x.item !== null), ingestBatch);
      setReport(r);
      if (r.imported > 0) onImported(r.imported);
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setBusy(false);
      setConfirming(false);
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
      aria-label={tt(lang, "import.fileAria", "Importar archivo CSV, JSON, JSONL o VDBDUMP")}
      onMouseDown={(e) => {
        if (e.target === e.currentTarget && !busy) onClose();
      }}
    >
      <section className="press-lg w-full max-w-2xl border-4 border-foreground bg-background">
        <header className="flex items-center gap-2 border-b-4 border-foreground bg-card px-4 py-3">
          <h2 className="font-display text-2xl text-stencil">IMPORT ARCHIVO</h2>
          <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
            {tp(lang, "import.maxMeta", "máx {m} registros · ns: {ns}", { m: String(MAX_IMPORT), ns: targetNs })}
          </span>
          <button
            className={`${btnBase} ml-auto bg-background`}
            onClick={onClose}
            disabled={busy}
            aria-label={tt(lang, "import.closeAria", "Cerrar")}
          >
            {tt(lang, "import.closeBtn", "✕ CERRAR")}
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
              {tp(lang, "import.reportDone", "✓ {n} importados", { n: String(report.imported) })}
            </p>
            {report.errors.length > 0 && (
              <ul className="mt-1 space-y-0.5">
                {report.errors.map((e, i) => (
                  <li key={i} className="font-tech text-[10px] text-muted-foreground">
                    {tp(lang, "import.reportRows", "filas {r}: {m}", { r: e.rows, m: e.message })}
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
              {tt(lang, "import.newFile", "↺ NUEVO ARCHIVO")}
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
                <span className="text-foreground">{tt(lang, "import.dropCta", "⤓ ARRASTRÁ UN ARCHIVO")}</span>
                <span>{tt(lang, "import.dropSub", ".csv · .json · .jsonl · .vdbdump — o hacé clic para elegir")}</span>
              </>
            )}
          </label>

          <div className="flex flex-wrap items-center gap-2">
            <label className="flex items-center gap-1 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              {tt(lang, "import.nsLabel", "namespace")}
              <input
                value={ns}
                onChange={(e) => {
                  setNs(e.target.value);
                  setConfirming(false);
                }}
                className="w-40 border-2 border-foreground bg-background px-1.5 py-0.5 font-tech text-[11px] outline-none focus:border-neon"
              />
            </label>
            <span className="ml-auto font-tech text-[10px] uppercase tracking-widest">
              {parsed ? (
                <>
                  <span className="text-foreground">{tp(lang, "import.validCount", "{v} ✓", { v: String(parsed.valid) })}</span>
                  <span className="text-muted-foreground">
                    {" "}
                    {tp(lang, "import.invalidCount", "· {i} ✗", { i: String(parsed.invalid) })} · {parsed.truncated ? tp(lang, "import.truncatedRows", "solo primeros {m}", { m: String(MAX_IMPORT) }) : tp(lang, "import.rowCount", "{n} filas", { n: String(rows.length) })}
                  </span>
                </>
              ) : (
                <span className="text-muted-foreground">{tt(lang, "import.noFile", "sin archivo")}</span>
              )}
            </span>
          </div>

          {parsed?.truncated && (
            <p className="font-tech text-[10px] text-accent-text">
              <TriangleAlert className="mr-1 inline h-3 w-3 align-[-1px]" strokeWidth={2.5} aria-hidden="true" />
              {tp(lang, "import.truncatedWarnFile", "el archivo supera {m} registros — se importan solo los primeros.", { m: String(MAX_IMPORT) })}
            </p>
          )}

          <div className="overflow-x-auto border-2 border-foreground">
            <table className="w-full border-collapse bg-background">
              <thead>
                <tr className="border-b-2 border-foreground font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                  <th className="px-2 py-1 text-left">#</th>
                  <th className="px-2 py-1 text-left">{tt(lang, "import.stateCol", "estado")}</th>
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
                        <span className="text-neon" aria-label={tt(lang, "import.validAria", "válida")}>
                          ✓
                        </span>
                      ) : (
                        <span className="text-foreground" title={r.error} aria-label={tt(lang, "import.invalidAria", "inválida")}>
                          ✗
                        </span>
                      )}
                    </td>
                    <td className="max-w-[120px] truncate px-2 py-1">{r.item?.id ?? <span className="text-muted-foreground">{tt(lang, "import.autoId", "auto")}</span>}</td>
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
                      {parsed ? tt(lang, "import.noRowsPaste", "sin filas parseables") : tt(lang, "import.noRowsFile", "el preview aparece al elegir un archivo")}
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
          {rows.length > PREVIEW_ROWS && (
            <p className="font-tech text-[10px] text-muted-foreground">
              {tp(lang, "import.moreRows", "… y {n} más (mostrando primeras {p})", { n: String(rows.length - PREVIEW_ROWS), p: String(PREVIEW_ROWS) })}
            </p>
          )}
        </div>

        <footer className="flex flex-wrap items-center gap-2 border-t-4 border-foreground bg-card px-4 py-3">
          {busy && (
            <span className="font-tech text-[10px] uppercase tracking-widest text-accent-text" role="status">
              {tt(lang, "import.importing", "importando…")}
            </span>
          )}
          {confirming && (
            <div role="alert" className="flex flex-wrap items-center gap-2 border-2 border-foreground bg-muted px-2 py-1.5">
              <span className="font-tech text-[10px] uppercase tracking-widest">
                {tp(lang, "import.confirmQ", "Importar {n} registros a ns “{ns}”?", { n: String(parsed?.valid ?? 0), ns: targetNs })}
              </span>
              <button
                type="button"
                onClick={() => void doImport()}
                disabled={busy}
                className="press border-2 border-foreground bg-neon px-2 py-1 text-[10px] font-bold text-background"
              >
                {tt(lang, "import.confirmBtn", "CONFIRMAR IMPORT")}
              </button>
              <button
                type="button"
                onClick={() => setConfirming(false)}
                disabled={busy}
                className="press border-2 border-foreground bg-background px-2 py-1 text-[10px]"
                aria-label={tt(lang, "import.cancelAria", "Cancelar importación")}
              >
                {tt(lang, "import.cancelBtn", "✕ CANCELAR")}
              </button>
            </div>
          )}
          <button
            className={`${btnBase} ml-auto bg-background`}
            onClick={handleImport}
            disabled={disabled || confirming}
            title={tt(lang, "import.chunksTitle", "Importar registros válidos en chunks de 50")}
          >
            {tp(lang, "import.runBtn", "⤓ IMPORTAR {n}", { n: String(parsed?.valid ?? 0) })}
          </button>
        </footer>
      </section>
    </div>
  );
}
// Export buttons (VS-16): JSONL of the current view + markdown status report.
// Manga styling (press, border-2, font-tech) matching the rest of Vanta Studio.
import { useMemo, useState } from "react";
import type { MemoryRecord } from "../../vanta";
import { list } from "../../vanta";
import { vantaErrorMessage } from "../../vanta";
import { recordsToJsonl, downloadText, copyText } from "./export-jsonl";
import { buildStatusReport } from "./statusReport";
import { tp, tt, type DesktopLang } from "../../i18n";
import { connectionPrefs } from "../../store/connections";

function filenameStamp(): string {
  return new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
}

export function ExportButtons({
  viewRecords,
  onError,
  onNotice = () => {},
  lang = connectionPrefs.get().lang ?? "es",
}: {
  /** Records currently visible in the grid (after column filters). */
  viewRecords: MemoryRecord[];
  onError: (msg: string) => void;
  onNotice?: (msg: string) => void;
  lang?: DesktopLang;
}) {
  const [busy, setBusy] = useState<"jsonl" | "report" | null>(null);
  const jsonl = useMemo(() => recordsToJsonl(viewRecords), [viewRecords]);

  async function handleJsonl(kind: "download" | "copy") {
    setBusy("jsonl");
    try {
      if (kind === "download") {
        downloadText(`vanta-view-${filenameStamp()}.jsonl`, jsonl);
        onNotice(tp(lang, "export.downloadedJsonl", "exportados {n} registros (JSONL)", { n: String(viewRecords.length) }));
      } else {
        const ok = await copyText(jsonl);
        onNotice(ok ? tp(lang, "export.copiedRecords", "copiados {n} registros (JSONL)", { n: String(viewRecords.length) }) : tt(lang, "export.clipboardGone", "clipboard unavailable"));
      }
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setBusy(null);
    }
  }

  async function handleReport(kind: "download" | "copy") {
    setBusy("report");
    try {
      // Report covers the whole store (counts/types/TTLs) — fetch up to a
      // sane cap; the view-only variant is the JSONL above.
      const all = await list({ limit: 500 });
      const md = buildStatusReport(all, {
        generatedAt: new Date().toISOString(),
        includeUpcomingTtls: true,
      });
      if (kind === "download") {
        downloadText(`vanta-report-${filenameStamp()}.md`, md);
        onNotice(
          all.length >= 500
            ? tt(lang, "export.reportSampled", "reporte generado (muestra: 500 — usá el grid para vistas más chicas)")
            : tp(lang, "export.reportGenerated", "reporte generado desde {n} registros (markdown)", { n: String(all.length) }),
        );
      } else {
        const ok = await copyText(md);
        onNotice(ok ? tt(lang, "export.reportCopied", "reporte copiado (markdown)") : tt(lang, "export.clipboardGone", "clipboard unavailable"));
      }
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setBusy(null);
    }
  }

  const btn =
    "press border-2 border-foreground bg-background px-2 py-1 text-[10px] font-semibold disabled:opacity-50";

  return (
    <div className="flex items-center gap-1">
      <button
        type="button"
        className={btn}
        disabled={busy !== null || viewRecords.length === 0}
        onClick={() => handleJsonl("download")}
        title={tp(lang, "export.jsonlTitle", "Descargar la vista actual ({n} registros) como JSONL importable", { n: String(viewRecords.length) })}
      >
        {busy === "jsonl" ? "…" : tt(lang, "export.jsonl", "⭳ JSONL")}
      </button>
      <button
        type="button"
        className={btn}
        disabled={busy !== null || viewRecords.length === 0}
        onClick={() => handleJsonl("copy")}
        title={tt(lang, "export.copyJsonlTitle", "Copiar la vista actual como JSONL")}
      >
        {busy === "jsonl" ? "…" : tt(lang, "export.copyJsonl", "⧉ copiar")}
      </button>
      <button
        type="button"
        className={btn}
        disabled={busy !== null}
        onClick={() => handleReport("download")}
        title={tt(lang, "export.reportTitle", "Descargar reporte de estado markdown (counts, tipos de metadata, TTLs)")}
      >
        {busy === "report" ? "…" : tt(lang, "export.report", "⭳ reporte")}
      </button>
      <button
        type="button"
        className={btn}
        disabled={busy !== null}
        onClick={() => handleReport("copy")}
        title={tt(lang, "export.copyReportTitle", "Copiar reporte de estado markdown")}
      >
        {busy === "report" ? "…" : tt(lang, "export.copyReport", "⧉ copiar reporte")}
      </button>
    </div>
  );
}
// Export buttons (VS-16): JSONL of the current view + markdown status report.
// Manga styling (press, border-2, font-tech) matching the rest of Vanta Studio.
import { useMemo, useState } from "react";
import type { MemoryRecord } from "../../vanta";
import { list } from "../../vanta";
import { vantaErrorMessage } from "../../vanta";
import { recordsToJsonl, downloadText, copyText } from "./export-jsonl";
import { buildStatusReport } from "./statusReport";

function filenameStamp(): string {
  return new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
}

export function ExportButtons({
  viewRecords,
  onError,
  onNotice = () => {},
}: {
  /** Records currently visible in the grid (after column filters). */
  viewRecords: MemoryRecord[];
  onError: (msg: string) => void;
  onNotice?: (msg: string) => void;
}) {
  const [busy, setBusy] = useState<"jsonl" | "report" | null>(null);
  const jsonl = useMemo(() => recordsToJsonl(viewRecords), [viewRecords]);

  async function handleJsonl(kind: "download" | "copy") {
    setBusy("jsonl");
    try {
      if (kind === "download") {
        downloadText(`vanta-view-${filenameStamp()}.jsonl`, jsonl);
        onNotice(`exported ${viewRecords.length} records (JSONL)`);
      } else {
        const ok = await copyText(jsonl);
        onNotice(ok ? `copied ${viewRecords.length} records (JSONL)` : "clipboard unavailable");
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
            ? `report generated (sampled: 500 — usa el grid para vistas más chicas)`
            : `report generated from ${all.length} records (markdown)`,
        );
      } else {
        const ok = await copyText(md);
        onNotice(ok ? "report copied (markdown)" : "clipboard unavailable");
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
        title={`Descargar la vista actual (${viewRecords.length} records) como JSONL importable`}
      >
        {busy === "jsonl" ? "…" : "⭳ JSONL"}
      </button>
      <button
        type="button"
        className={btn}
        disabled={busy !== null || viewRecords.length === 0}
        onClick={() => handleJsonl("copy")}
        title="Copiar la vista actual como JSONL"
      >
        {busy === "jsonl" ? "…" : "⧉ copiar"}
      </button>
      <button
        type="button"
        className={btn}
        disabled={busy !== null}
        onClick={() => handleReport("download")}
        title="Descargar reporte de estado markdown (counts, tipos de metadata, TTLs)"
      >
        {busy === "report" ? "…" : "⭳ reporte"}
      </button>
      <button
        type="button"
        className={btn}
        disabled={busy !== null}
        onClick={() => handleReport("copy")}
        title="Copiar reporte de estado markdown"
      >
        {busy === "report" ? "…" : "⧉ copiar reporte"}
      </button>
    </div>
  );
}
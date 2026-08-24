// Snapshot export (ADMIN-09). Reads the latest vanta_metrics snapshot from the
// shared poll (useMetricsPoll, DESKTOP-29) and persists it to localStorage so
// the app can show last-run data before the live poll responds. Export
// downloads the snapshot as JSON via blob (no Tauri fs/dialog plugin installed
// — frontend download is the contract minimum).
import { useEffect, useState } from "react";
import { OperationalMetrics } from "../vanta";
import { useMetricsPoll } from "../hooks/useMetricsPoll";

const LS_KEY = "vanta.last_snapshot";

interface Stored {
  at: number;
  snapshot: OperationalMetrics;
}

function loadStored(): Stored | null {
  try {
    const raw = localStorage.getItem(LS_KEY);
    if (!raw) return null;
    const p: unknown = JSON.parse(raw);
    if (p && typeof p === "object" && typeof (p as Stored).at === "number" && (p as Stored).snapshot) {
      return p as Stored;
    }
  } catch {
    // Corrupt entry — ignore and fall through to "no snapshot yet".
  }
  return null;
}

function fileName(at: number): string {
  const ts = new Date(at).toISOString().replace(/[:.]/g, "-");
  return `vanta-snapshot-${ts}.json`;
}

export default function ExportPanel() {
  const [stored, setStored] = useState<Stored | null>(() => loadStored());
  const { history, error } = useMetricsPoll();
  const live = history[history.length - 1] ?? null;

  // Persist each new live snapshot so the next run has last-run data.
  useEffect(() => {
    if (!live) return;
    const rec: Stored = { at: Date.now(), snapshot: live };
    localStorage.setItem(LS_KEY, JSON.stringify(rec));
    setStored(rec);
  }, [live]);

  // Live snapshot wins; before the poll answers, fall back to last persisted run.
  const snapshot = live ?? stored?.snapshot ?? null;
  const savedAt = stored?.at ?? null;

  function download() {
    if (!snapshot) return;
    const blob = new Blob([JSON.stringify(snapshot, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = fileName(savedAt ?? Date.now());
    a.click();
    URL.revokeObjectURL(url);
  }

  return (
    <section
      aria-label="Exportar snapshot"
      className="border-[3px] border-foreground bg-card p-4 shadow-ink"
    >
      <div className="mb-3 flex items-center justify-between gap-2">
        <h2 className="m-0 font-tech text-xs uppercase tracking-widest">Exportar Snapshot</h2>
        <span className="text-muted-foreground">
          {error ? "snapshot no disponible — reintentando…" : "última corrida de vanta_metrics"}
        </span>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <button
          type="button"
          onClick={download}
          disabled={!snapshot}
          className="press cursor-pointer border-2 border-foreground bg-background px-2.5 py-1.5 text-sm disabled:cursor-default disabled:opacity-50"
        >
          Exportar snapshot
        </button>
        <span
          className={`font-tech text-xs ${snapshot ? "text-foreground" : "text-muted-foreground"}`}
          data-status={snapshot ? "ok" : "idle"}
        >
          {savedAt ? `guardado: ${new Date(savedAt).toLocaleString()}` : "sin snapshot aún"}
        </span>
      </div>
    </section>
  );
}

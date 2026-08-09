// Snapshot export (ADMIN-09). Fetches the last vanta_metrics snapshot and
// persists it to localStorage so the app can show last-run data before the
// live poll responds. Export downloads the snapshot as JSON via blob (no Tauri
// fs/dialog plugin installed — frontend download is the contract minimum).
import { useEffect, useState } from "react";
import { metrics, OperationalMetrics, vantaErrorMessage } from "../vanta";

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
  const [live, setLive] = useState<OperationalMetrics | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    metrics()
      .then((m) => {
        if (!alive) return;
        setLive(m);
        setError(null);
        const rec: Stored = { at: Date.now(), snapshot: m };
        localStorage.setItem(LS_KEY, JSON.stringify(rec));
        setStored(rec);
      })
      .catch((e) => {
        if (alive) setError(vantaErrorMessage(e));
      });
    return () => {
      alive = false;
    };
  }, []);

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
    <section className="panel export" aria-label="Snapshot export">
      <div className="panel-head">
        <h2>Export Snapshot</h2>
        <span className="muted">{error ? error : "last vanta_metrics run"}</span>
      </div>
      <div className="export-row">
        <button onClick={download} disabled={!snapshot}>
          Export snapshot
        </button>
        <span className="export-saved" data-status={snapshot ? "ok" : "idle"}>
          {savedAt ? `last saved: ${new Date(savedAt).toLocaleString()}` : "no snapshot yet"}
        </span>
      </div>
    </section>
  );
}

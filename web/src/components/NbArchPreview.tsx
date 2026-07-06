import { useState, useCallback } from "react";
import { NbSection, NbSectionHeader } from "../components/nb";
import "../styles/arch-preview.css";

const LAYERS = [
  {
    id: "sdk",
    label: "SDK LAYER",
    items: "PyO3 · Rust · C API",
    detail:
      "Three binding surfaces covering every integration path. Python via PyO3 for data scientists, native Rust for systems, C ABI for interop.",
  },
  {
    id: "query",
    label: "QUERY ENGINE",
    items: "SQL · HNSW · BM25",
    detail:
      "Triple-index architecture. HNSW for ANN, BM25 for FTS, intelligent router fusing scores into ranked results.",
  },
  {
    id: "opt",
    label: "OPTIMIZER",
    items: "Cost-based planner",
    detail:
      "Cost-based planning across vector, text, SQL predicates. Auto-selects index vs sequential scan.",
  },
  {
    id: "wal",
    label: "WAL",
    items: "Write-Ahead Log",
    detail:
      "Crash-safe durability. Every write is logged before commit. Automatic recovery on restart.",
  },
  {
    id: "store",
    label: "STORE",
    items: "In-Memory · SQLite VFS",
    detail:
      "Dual-mode: in-memory for sub-µs reads, SQLite VFS for single-file persistence. Configurable buffer pool.",
  },
  {
    id: "io",
    label: "IO LAYER",
    items: "io_uring · mmap",
    detail:
      "Async I/O with io_uring (Linux), overlapped I/O (Win). Memory-mapped regions for zero-copy reads.",
  },
];

export function NbArchPreview() {
  const [selected, setSelected] = useState<number | null>(null);
  const toggle = useCallback((i: number) => setSelected((p) => (p === i ? null : i)), []);

  return (
    <NbSection ariaLabel="Architecture">
      <NbSectionHeader
        monoLabel="[STACK]"
        headline="Six layers. One process."
        sub="From SDK to disk — compiled to native, running in your address space."
      />

      {/* 3D Layer Stack */}
      <div className="nb-stack-3d" role="img" aria-label="3D architecture layer stack">
        {LAYERS.map((layer, i) => {
          const depth = LAYERS.length - 1 - i;
          const zOffset = depth * 12;
          const isSel = selected === i;

          return (
            <button
              key={layer.id}
              type="button"
              className={`nb-stack-layer ${isSel ? "nb-stack-layer--sel" : ""} ${selected !== null && !isSel ? "nb-stack-layer--dim" : ""}`}
              style={{ zIndex: depth + 1, "--layer-z": `${zOffset}px` } as React.CSSProperties}
              onClick={() => toggle(i)}
              aria-pressed={isSel}
            >
              <span className="nb-stack-layer-num">{String(i + 1).padStart(2, "0")}</span>
              <div className="nb-stack-layer-content">
                <span className="nb-stack-layer-label">{layer.label}</span>
                <span className="nb-stack-layer-items">{layer.items}</span>
              </div>
              <span className="nb-stack-layer-arrow">{">"}</span>
            </button>
          );
        })}
      </div>

      {/* Detail panel */}
      <div
        className={`nb-stack-detail ${selected !== null ? "nb-stack-detail--show" : ""}`}
        aria-hidden={selected === null}
      >
        {selected !== null && <p className="nb-stack-detail-text">{LAYERS[selected].detail}</p>}
      </div>
    </NbSection>
  );
}

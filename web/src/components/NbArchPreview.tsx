import { useState } from "react";
import "../styles/arch-preview.css";

const LAYERS = [
  {
    id: "clients",
    label: "Clients",
    items: "Python SDK (PyO3) | Rust SDK | C API",
    detail:
      "Three binding surfaces covering every integration path. Python via PyO3 for data scientists, native Rust crate for systems programmers, and C ABI for language interop.",
  },
  {
    id: "query-engine",
    label: "Query Engine",
    items: "SQL Parser | Vector Index (HNSW) | BM25 FTS",
    detail:
      "Triple-index architecture. HNSW for approximate nearest neighbor, BM25 for full-text relevance, and an intelligent router that fuses both scores into a single ranked result.",
  },
  {
    id: "optimizer",
    label: "Optimizer",
    items: "Cost-based Query Optimizer",
    detail:
      "Cost-based query planning across vector, text, and SQL predicates. Automatically chooses index scan vs sequential scan based on selectivity estimates.",
  },
  {
    id: "storage",
    label: "Storage",
    items: "Write-Ahead Log (WAL) | In-Memory Store",
    detail:
      "Dual-mode architecture. In-memory store for sub-microsecond reads, WAL for crash-safe durability. Configurable buffer pool and page size.",
  },
  {
    id: "persistence",
    label: "Persistence",
    items: "SQLite VFS + DuckDB + Custom Backends",
    detail:
      "Pluggable backend layer. Default SQLite VFS for single-file simplicity, DuckDB for analytical workloads, or custom backends for specialized storage.",
  },
  {
    id: "io-layer",
    label: "IO Layer",
    items: "Disk I/O | Memory-mapped | Network (optional)",
    detail:
      "Async I/O with io_uring (Linux) and overlapped I/O (Windows). Memory-mapped regions for zero-copy reads. Optional networked mode for remote storage.",
  },
];

export function NbArchPreview() {
  const [selectedLayer, setSelectedLayer] = useState<number | null>(null);

  const handleSelect = (index: number) => {
    setSelectedLayer((prev) => (prev === index ? null : index));
  };

  return (
    <section className="nb-section" aria-label="Architecture">
      <div className="nb-inner">
        <div className="nb-section-header">
          <h2 className="nb-amber-title">Architecture</h2>
        </div>
        <hr className="nb-divider" />

        <div className="nb-arch-stack">
          {LAYERS.map((layer, i) => {
            const isSelected = selectedLayer === i;
            return (
              <div key={layer.id} className="nb-arch-layer-wrap">
                <button
                  type="button"
                  className={`nb-arch-layer${isSelected ? " nb-arch-layer--selected" : ""}`}
                  onClick={() => handleSelect(i)}
                  aria-expanded={isSelected}
                >
                  <span className="nb-arch-layer-label">{layer.label}</span>
                  <span className="nb-arch-layer-items">{layer.items}</span>
                  <span className="nb-arch-layer-arrow">{isSelected ? "▲" : "▼"}</span>
                </button>
                <div
                  className={`nb-arch-detail${isSelected ? " nb-arch-detail--open" : ""}`}
                  role="region"
                  aria-hidden={!isSelected}
                >
                  <div className="nb-arch-detail-inner">
                    <p className="nb-arch-detail-text">{layer.detail}</p>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}

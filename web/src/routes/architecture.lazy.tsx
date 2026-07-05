import { createLazyRoute } from "@tanstack/react-router";
import { useState } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/architecture.css";

export const Route = createLazyRoute("/architecture")({
  component: ArchitecturePage,
  pendingComponent: PendingComponent,
});

function SpecRow({ label, val, desc }: { label: string; val: string; desc: string }) {
  return (
    <tr>
      <td className="architecture-spec-label">{label}</td>
      <td className="architecture-spec-value">{val}</td>
      <td className="architecture-spec-desc">{desc}</td>
    </tr>
  );
}

function PerformanceProfiler() {
  const [hoveredSegment, setHoveredSegment] = useState<string | null>(null);

  const segments = [
    {
      id: "ffi",
      label: "FFI Boundary Translation",
      share: 12,
      desc: "Fast PyO3 compilation translating Python dictionaries to native Rust structs with zero memory copies.",
      color: "var(--steel)",
    },
    {
      id: "planner",
      label: "Query Optimization",
      share: 24,
      desc: "Sifts through filters, sets up lexical scanning ranges, and prepares candidate HNSW entry points.",
      color: "var(--muted)",
    },
    {
      id: "index",
      label: "HNSW & BM25 Traversal",
      share: 48,
      desc: "Core path searching approximate nearest neighbors and evaluating index statistics in parallel.",
      color: "var(--amber)",
    },
    {
      id: "wal",
      label: "WAL Commit & fsync",
      share: 16,
      desc: "Forces durability logging, computes transaction CRC32C, and syncs sectors on disks.",
      color: "var(--foreground)",
    },
  ];

  const hoveredData = segments.find((s) => s.id === hoveredSegment);

  return (
    <div className="nb-frame">
      <div className="architecture-profiler-header">
        <span className="architecture-profiler-title">CPU TIME DISTRIBUTION</span>
        <span className="architecture-profiler-hint">HOVER SEGMENTS TO PROFILE</span>
      </div>

      <div className="architecture-profiler-bar">
        {segments.map((seg) => (
          <div
            key={seg.id}
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: "pointer",
              width: `${seg.share}%`,
              height: "100%",
              background: seg.id === hoveredSegment ? "var(--amber)" : seg.color,
              transition: "background-color 150ms var(--ease-cut)",
            }}
            onMouseEnter={() => setHoveredSegment(seg.id)}
            onMouseLeave={() => setHoveredSegment(null)}
          >
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "0.65rem",
                fontWeight: 700,
                color: seg.id === hoveredSegment ? "#000000" : "var(--background)",
                opacity: seg.share > 15 ? 1 : 0,
                fontVariantNumeric: "tabular-nums",
              }}
            >
              {seg.share}%
            </span>
          </div>
        ))}
      </div>

      <div className="architecture-profiler-detail">
        {hoveredData ? (
          <div>
            <div className="architecture-hovered-title">
              {hoveredData.label} — {hoveredData.share}% of query budget
            </div>
            <p className="architecture-hovered-desc">{hoveredData.desc}</p>
          </div>
        ) : (
          <div>
            <div className="architecture-default-title">Engine Performance Summary</div>
            <p className="architecture-default-desc">
              Hover over the latency bar segments above to analyze where the database spends CPU
              cycles during typical multi-modal queries.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

function ArchitecturePage() {
  const layers = [
    {
      num: "01",
      tag: "Python & Rust bindings",
      title: "Zero-Copy SDK",
      body: "High-level importable libraries. Wraps raw Rust structures via PyO3, translating namespaces, parameters, and query lists without GIL overhead.",
    },
    {
      num: "02",
      tag: "Compiles to native library",
      title: "Stable FFI Boundary",
      body: "A clean FFI contract with zero IPC, sockets, or network overhead. Memory pointers are shared directly between Python interpreter memory and Rust heap.",
    },
    {
      num: "03",
      tag: "Multi-modal planning",
      title: "Core Search Optimizer",
      body: "Analyzes filtering conditions and queries. Directs traffic to keyword databases and HNSW indexing systems, blending results through fast RRF fusers.",
    },
    {
      num: "04",
      tag: "Log-Structured persist",
      title: "Storage Engine",
      body: "Coordinates active memory buffers, Write-Ahead Logs, block sync thresholds, and backgrounds compaction passes directly into a single database file.",
    },
  ];

  return (
    <div>
      <NbSubpageHero
        num="02"
        title={
          <span>
            Direct compilation.
            <br />
            Shared-memory execution.
          </span>
        }
        sub="VantaDB compiles to a single native library wrapper. No extra TCP connections, no separate processes to start. Just pure memory speed."
      />

      <main>
        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-asymmetric">
              <div>
                <h2 className="architecture-section-title">Stack Layers</h2>
                <p className="architecture-section-desc">
                  VantaDB provides safe bindings on top of a highly optimized multi-modal execution
                  core and storage layer.
                </p>
              </div>
              <div className="nb-grid nb-grid--cols-2">
                {layers.map((lyr) => (
                  <div key={lyr.num} className="nb-cell architecture-layer-cell">
                    <div className="architecture-layer-header">
                      <span className="architecture-layer-tag">{lyr.tag}</span>
                      <span className="architecture-layer-num">LAYER {lyr.num}</span>
                    </div>
                    <h3 className="architecture-layer-title">{lyr.title}</h3>
                    <p className="architecture-layer-body">{lyr.body}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </section>

        <div className="nb-divider" />

        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-asymmetric--right">
              <div>
                <PerformanceProfiler />
              </div>
              <div>
                <h2 className="architecture-section-title">Query Latency</h2>
                <p className="architecture-section-desc architecture-section-desc--compact">
                  Due to direct sharing of pointer addresses, the cost of crossing FFI bindings is
                  less than 12% of total search time, leaving CPU resources free to evaluate
                  similarity indexes.
                </p>
              </div>
            </div>
          </div>
        </section>

        <div className="nb-divider" />

        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-asymmetric">
              <div>
                <h2 className="architecture-section-title">Operational Limits</h2>
                <p className="architecture-section-desc architecture-section-desc--compact">
                  Technical limits enforced at memory layer boundaries to prevent out-of-memory
                  states during heavy concurrent query evaluations.
                </p>
              </div>
              <div>
                <div className="nb-frame">
                  <table className="nb-table architecture-table">
                    <thead>
                      <tr>
                        <th>Parameter</th>
                        <th>Limits</th>
                        <th>Details</th>
                      </tr>
                    </thead>
                    <tbody>
                      <SpecRow
                        label="Key Size Limit"
                        val="1 KB"
                        desc="Identifiers must be compact to ensure lightning-fast pointer indexing in standard LSM memory bounds."
                      />
                      <SpecRow
                        label="Vector Dimensions Limit"
                        val="32,768 dims"
                        desc="Configurable HNSW index structures, optimized up to high-end sparse embeddings."
                      />
                      <SpecRow
                        label="Text Content Limit"
                        val="10 MB per put"
                        desc="Larger chunk payloads are offloaded directly to binary buffers, preserving planner heap latency floors."
                      />
                      <SpecRow
                        label="Metadata Size Limit"
                        val="64 KB"
                        desc="Structured dictionaries for lexical pre-filtering operations before RRF fusion runs."
                      />
                      <SpecRow
                        label="Concurrency Model"
                        val="1 Writer · ∞ Readers"
                        desc="Safe thread concurrent read executions, write paths synchronize locks to protect journal logs."
                      />
                      <SpecRow
                        label="sync_mode Options"
                        val="always | periodic | never"
                        desc="Set flush WAL constraints depending on durability trade-offs (Periodic sync defaults to 500ms intervals)."
                      />
                      <SpecRow
                        label="Engine States"
                        val="Init → Ready → Flush → Closed"
                        desc="Exposes state transitions cleanly through FFI to allow hot rebuilding without data losses."
                      />
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="nb-section nb-bg-dot">
          <div className="nb-inner">
            <div className="nb-block-amber">
              <div className="architecture-cta-wrap">
                <div>
                  <h2 className="architecture-cta-title">
                    Compiles to native. Runs in your process.
                  </h2>
                  <p className="architecture-cta-sub">Install VantaDB in one command.</p>
                </div>
                <code className="architecture-cta-code">pip install vantadb-py</code>
              </div>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}

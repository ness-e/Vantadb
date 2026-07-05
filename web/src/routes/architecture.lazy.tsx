import { createLazyRoute } from "@tanstack/react-router";
import { useState } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/architecture")({
  component: ArchitecturePage,
  pendingComponent: PendingComponent,
});

function SpecRow({ label, val, desc }: { label: string; val: string; desc: string }) {
  return (
    <tr>
      <td
        style={{
          fontFamily: "var(--font-mono)",
          fontWeight: 700,
          fontSize: "0.7rem",
          letterSpacing: "0.05em",
          color: "var(--foreground)",
          border: "2px solid var(--border-visible)",
          padding: "var(--space-md)",
        }}
      >
        {label}
      </td>
      <td
        style={{
          fontFamily: "var(--font-mono)",
          fontWeight: 700,
          fontSize: "0.7rem",
          color: "var(--amber)",
          border: "2px solid var(--border-visible)",
          padding: "var(--space-md)",
        }}
      >
        {val}
      </td>
      <td
        style={{
          fontFamily: "var(--font-sans)",
          fontSize: "var(--text-code)",
          color: "var(--muted)",
          lineHeight: 1.6,
          border: "2px solid var(--border-visible)",
          padding: "var(--space-md)",
        }}
      >
        {desc}
      </td>
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
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "baseline",
          marginBottom: "var(--space-lg)",
        }}
      >
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "0.65rem",
            color: "var(--steel)",
            textTransform: "uppercase",
            letterSpacing: "0.05em",
          }}
        >
          CPU TIME DISTRIBUTION
        </span>
        <span
          style={{ fontFamily: "var(--font-mono)", fontSize: "0.65rem", color: "var(--muted)" }}
        >
          HOVER SEGMENTS TO PROFILE
        </span>
      </div>

      <div
        style={{
          display: "flex",
          width: "100%",
          height: "2rem",
          gap: "1px",
          marginBottom: "var(--space-xl)",
          background: "var(--border)",
        }}
      >
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

      <div
        style={{
          paddingTop: "var(--space-lg)",
          minHeight: "5.5rem",
          borderTop: "1px solid var(--border)",
        }}
      >
        {hoveredData ? (
          <div>
            <div
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "0.7rem",
                fontWeight: 700,
                textTransform: "uppercase",
                color: "var(--amber)",
                letterSpacing: "0.08em",
                marginBottom: "var(--space-2xs)",
                fontVariantNumeric: "tabular-nums",
              }}
            >
              {hoveredData.label} — {hoveredData.share}% of query budget
            </div>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "0.78rem",
                color: "var(--muted)",
                margin: 0,
                lineHeight: 1.6,
              }}
            >
              {hoveredData.desc}
            </p>
          </div>
        ) : (
          <div>
            <div
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "0.7rem",
                fontWeight: 700,
                textTransform: "uppercase",
                color: "var(--steel)",
                letterSpacing: "0.08em",
                marginBottom: "var(--space-2xs)",
              }}
            >
              Engine Performance Summary
            </div>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "0.78rem",
                color: "var(--muted)",
                margin: 0,
                lineHeight: 1.6,
              }}
            >
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
                <h2
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "clamp(2rem,4vw,3rem)",
                    fontWeight: 800,
                    letterSpacing: "-0.04em",
                    marginTop: "var(--space-lg)",
                    marginBottom: "var(--space-lg)",
                    lineHeight: 1.25,
                  }}
                >
                  Stack Layers
                </h2>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.95rem",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                  }}
                >
                  VantaDB provides safe bindings on top of a highly optimized multi-modal execution
                  core and storage layer.
                </p>
              </div>
              <div className="nb-grid nb-grid--cols-2">
                {layers.map((lyr) => (
                  <div
                    key={lyr.num}
                    className="nb-cell"
                    style={{ display: "flex", flexDirection: "column", gap: "var(--space-xs)" }}
                  >
                    <div
                      style={{
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "baseline",
                      }}
                    >
                      <span
                        style={{
                          fontFamily: "var(--font-mono)",
                          fontSize: "0.6rem",
                          color: "var(--steel)",
                          textTransform: "uppercase",
                          letterSpacing: "0.05em",
                        }}
                      >
                        {lyr.tag}
                      </span>
                      <span
                        style={{
                          fontFamily: "var(--font-mono)",
                          fontSize: "0.6rem",
                          color: "var(--amber)",
                          fontWeight: 700,
                        }}
                      >
                        LAYER {lyr.num}
                      </span>
                    </div>
                    <h3
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "1.25rem",
                        lineHeight: "1.75rem",
                        fontWeight: 700,
                        marginTop: "var(--space-xs)",
                        marginBottom: 0,
                        color: "var(--foreground)",
                      }}
                    >
                      {lyr.title}
                    </h3>
                    <p
                      style={{
                        fontSize: "var(--text-code)",
                        color: "var(--muted)",
                        lineHeight: 1.6,
                        margin: 0,
                      }}
                    >
                      {lyr.body}
                    </p>
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
                <h2
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "clamp(2rem,4vw,3rem)",
                    fontWeight: 800,
                    letterSpacing: "-0.04em",
                    marginTop: "var(--space-lg)",
                    marginBottom: "var(--space-lg)",
                    lineHeight: 1.25,
                  }}
                >
                  Query Latency
                </h2>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.95rem",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    marginBottom: 0,
                  }}
                >
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
                <h2
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "clamp(2rem,4vw,3rem)",
                    fontWeight: 800,
                    letterSpacing: "-0.04em",
                    marginTop: "var(--space-lg)",
                    marginBottom: "var(--space-lg)",
                    lineHeight: 1.25,
                  }}
                >
                  Operational Limits
                </h2>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.95rem",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    marginBottom: 0,
                  }}
                >
                  Technical limits enforced at memory layer boundaries to prevent out-of-memory
                  states during heavy concurrent query evaluations.
                </p>
              </div>
              <div>
                <div className="nb-frame">
                  <table className="nb-table" style={{ border: "none" }}>
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
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  gap: "var(--space-md)",
                  flexWrap: "wrap",
                }}
              >
                <div>
                  <h2
                    style={{
                      fontFamily: "var(--font-display)",
                      fontSize: "1.5rem",
                      lineHeight: "2rem",
                      fontWeight: 800,
                      color: "var(--text-on-amber)",
                    }}
                  >
                    Compiles to native. Runs in your process.
                  </h2>
                  <p style={{ fontSize: "0.875rem", color: "var(--text-on-amber)", opacity: 0.8 }}>
                    Install VantaDB in one command.
                  </p>
                </div>
                <code
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "1.125rem",
                    lineHeight: "1.75rem",
                    fontWeight: 700,
                    color: "var(--text-on-amber)",
                  }}
                >
                  pip install vantadb-py
                </code>
              </div>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}

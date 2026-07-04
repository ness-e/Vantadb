import { createLazyRoute } from "@tanstack/react-router";
import { useState } from "react";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/architecture")({
  component: ArchitecturePage,
  pendingComponent: PendingComponent,
});

function SpecRow({ label, val, desc }: { label: string; val: string; desc: string }) {
  return (
    <tr className="spec-table-row">
      <td
        className="spec-table-cell spec-table-label font-mono font-bold text-[0.7rem] tracking-[0.05em] text-foreground"
      >
        {label}
      </td>
      <td
        className="spec-table-cell spec-table-value font-mono font-bold text-[0.7rem] text-[var(--amber)]"
      >
        {val}
      </td>
      <td
        className="spec-table-desc font-sans text-[0.82rem] text-[var(--muted)] leading-[1.5]"
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
    <div
      className="border border-[var(--border)] p-10 bg-[var(--surface)]"
    >
      <div
        className="flex justify-between items-baseline mb-6"
      >
        <span
          className="font-mono text-[0.65rem] text-[var(--steel)] uppercase tracking-[0.05em]"
        >
          CPU TIME DISTRIBUTION
        </span>
        <span
          className="font-mono text-[0.65rem] text-[var(--muted)]"
        >
          HOVER SEGMENTS TO PROFILE
        </span>
      </div>

      <div
        className="flex w-full h-8 bg-[var(--border)] gap-[1px] mb-8"
      >
        {segments.map((seg) => (
          <div
            key={seg.id}
            className="flex items-center justify-center cursor-pointer"
            style={{
              width: `${seg.share}%`,
              height: "100%",
              background: seg.id === hoveredSegment ? "var(--amber)" : seg.color,
              transition: "background-color 150ms var(--ease-cut)",
            }}
            onMouseEnter={() => setHoveredSegment(seg.id)}
            onMouseLeave={() => setHoveredSegment(null)}
          >
            <span
              className="font-mono text-[0.65rem] font-bold"
              style={{
                color:
                  seg.id === hoveredSegment
                    ? "#000000"
                    : seg.id === "wal"
                      ? "var(--background)"
                      : "var(--background)",
                opacity: seg.share > 15 ? 1 : 0,
              }}
            >
              {seg.share}%
            </span>
          </div>
        ))}
      </div>

      <div
        className="border-t border-[var(--subtle)] pt-5 min-h-[5.5rem]"
      >
        {hoveredData ? (
          <div>
            <div
              className="font-mono text-[0.7rem] font-bold uppercase text-[var(--amber)] tracking-[0.08em] mb-1"
            >
              {hoveredData.label} — {hoveredData.share}% of query budget
            </div>
            <p
              className="font-sans text-[0.78rem] text-[var(--muted)] m-0 leading-[1.5]"
            >
              {hoveredData.desc}
            </p>
          </div>
        ) : (
          <div>
            <div
              className="font-mono text-[0.7rem] font-bold uppercase text-[var(--steel)] tracking-[0.08em] mb-1"
            >
              Engine Performance Summary
            </div>
            <p
              className="font-sans text-[0.78rem] text-[var(--muted)] m-0 leading-[1.5]"
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
    <div className="swiss-page">
      <SwissSubpageHero
        num="02"
        eyebrow="Architecture"
        title={
          <span>
            Direct compilation.
            <br />
            Shared-memory execution.
          </span>
        }
        sub="VantaDB compiles to a single native library wrapper. No extra TCP connections, no separate processes to start. Just pure memory speed."
      />

      <main className="swiss-main">
        <section className="swiss-page-section swiss-page-section--bordered">
          <div className="swiss-grid-12 items-start">
            <div className="col-span-4">
              <span className="swiss-eyebrow">01 / 03 — The Stack</span>
              <h2
                className="font-display text-[clamp(2rem,4vw,3rem)] font-extrabold tracking-[-0.04em] my-5 leading-[1.05]"
              >
                Stack Layers
              </h2>
              <p
                className="font-sans text-[0.95rem] text-[var(--muted)] leading-[1.6]"
              >
                VantaDB provides safe bindings on top of a highly optimized multi-modal execution
                core and storage layer.
              </p>
            </div>

            <div
              className="col-span-8 grid grid-cols-2 gap-[1px] bg-[var(--border)] border border-[var(--border)]"
            >
              {layers.map((lyr) => (
                <div
                  key={lyr.num}
                  className="bg-background px-8 py-10 flex flex-col gap-2"
                >
                  <div
                    className="flex justify-between items-baseline"
                  >
                    <span
                      className="font-mono text-[0.6rem] text-[var(--steel)] uppercase tracking-[0.05em]"
                    >
                      {lyr.tag}
                    </span>
                    <span
                      className="font-mono text-[0.6rem] text-[var(--amber)] font-bold"
                    >
                      LAYER {lyr.num}
                    </span>
                  </div>
                  <h3
                    className="font-display text-xl font-bold mt-2 mb-0 text-foreground"
                  >
                    {lyr.title}
                  </h3>
                  <p
                    className="text-[0.82rem] text-[var(--muted)] leading-[1.5] m-0"
                  >
                    {lyr.body}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="swiss-page-section swiss-page-section--bordered">
          <div className="swiss-grid-12 items-start">
            <div className="col-span-8">
              <PerformanceProfiler />
            </div>
            <div className="col-span-4">
              <span className="swiss-eyebrow">02 / 03 — Profiling</span>
              <h2
                className="font-display text-[clamp(2rem,4vw,3rem)] font-extrabold tracking-[-0.04em] my-5 leading-[1.05]"
              >
                Query Latency
              </h2>
              <p
                className="font-sans text-[0.95rem] text-[var(--muted)] leading-[1.6] mb-0"
              >
                Due to direct sharing of pointer addresses, the cost of crossing FFI bindings is
                less than 12% of total search time, leaving CPU resources free to evaluate
                similarity indexes.
              </p>
            </div>
          </div>
        </section>

        <section className="swiss-page-section">
          <div className="swiss-grid-12 items-start">
            <div className="col-span-4">
              <span className="swiss-eyebrow">03 / 03 — Specifications</span>
              <h2
                className="font-display text-[clamp(2rem,4vw,3rem)] font-extrabold tracking-[-0.04em] my-5 leading-[1.05]"
              >
                Operational Limits
              </h2>
              <p
                className="font-sans text-[0.95rem] text-[var(--muted)] leading-[1.6] mb-0"
              >
                Technical limits enforced at memory layer boundaries to prevent out-of-memory states
                during heavy concurrent query evaluations.
              </p>
            </div>

            <div className="col-span-8">
              <div className="border border-[var(--border)] overflow-hidden">
                <table
                  className="arch-spec-table w-full border-collapse"
                >
                  <thead>
                    <tr
                      className="bg-[var(--surface)] border-b border-[var(--border)]"
                    >
                      <th
                        className="px-4 py-5 font-mono text-[0.65rem] font-bold uppercase text-[var(--steel)] text-left"
                      >
                        Parameter
                      </th>
                      <th
                        className="px-4 py-5 font-mono text-[0.65rem] font-bold uppercase text-[var(--steel)] text-left"
                      >
                        Limits
                      </th>
                      <th
                        className="px-4 py-5 font-mono text-[0.65rem] font-bold uppercase text-[var(--steel)] text-left"
                      >
                        Details
                      </th>
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
        </section>
      </main>

      <style>{`
        @media (max-width: 768px) {
          .swiss-grid-12 { grid-template-columns: 1fr !important; }
          .arch-spec-table { font-size: 0.7rem !important; }
        }
      `}</style>
    </div>
  );
}



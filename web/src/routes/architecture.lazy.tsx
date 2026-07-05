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
    <tr>
      <td className="font-mono font-bold text-[0.7rem] tracking-[0.05em] text-foreground border-2 border-[var(--border-visible)] px-4 py-4">
        {label}
      </td>
      <td className="font-mono font-bold text-[0.7rem] text-amber border-2 border-[var(--border-visible)] px-4 py-4">
        {val}
      </td>
      <td className="font-sans text-[0.82rem] text-muted leading-relaxed border-2 border-[var(--border-visible)] px-4 py-4">
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
    <div className="nb-frame" data-frame-label="CPU TIME DISTRIBUTION">
      <div className="flex justify-between items-baseline mb-6">
        <span className="font-mono text-[0.65rem] text-steel uppercase tracking-[0.05em]">
          CPU TIME DISTRIBUTION
        </span>
        <span className="font-mono text-[0.65rem] text-muted">HOVER SEGMENTS TO PROFILE</span>
      </div>

      <div className="flex w-full h-8 gap-[1px] mb-8" style={{ background: "var(--border)" }}>
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
                color: seg.id === hoveredSegment ? "#000000" : "var(--background)",
                opacity: seg.share > 15 ? 1 : 0,
              }}
            >
              {seg.share}%
            </span>
          </div>
        ))}
      </div>

      <div className="pt-5 min-h-[5.5rem]" style={{ borderTop: "1px solid var(--border)" }}>
        {hoveredData ? (
          <div>
            <div className="font-mono text-[0.7rem] font-bold uppercase text-amber tracking-[0.08em] mb-1">
              {hoveredData.label} — {hoveredData.share}% of query budget
            </div>
            <p className="font-sans text-[0.78rem] text-muted m-0 leading-relaxed">
              {hoveredData.desc}
            </p>
          </div>
        ) : (
          <div>
            <div className="font-mono text-[0.7rem] font-bold uppercase text-steel tracking-[0.08em] mb-1">
              Engine Performance Summary
            </div>
            <p className="font-sans text-[0.78rem] text-muted m-0 leading-relaxed">
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

      <main>
        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-asymmetric">
              <div>
                <div className="nb-label">01 / 03 — The Stack</div>
                <h2 className="font-display text-[clamp(2rem,4vw,3rem)] font-extrabold tracking-[-0.04em] my-5 leading-tight">
                  Stack Layers
                </h2>
                <p className="font-sans text-[0.95rem] text-muted leading-relaxed">
                  VantaDB provides safe bindings on top of a highly optimized multi-modal execution
                  core and storage layer.
                </p>
              </div>
              <div className="nb-grid nb-grid--cols-2">
                {layers.map((lyr) => (
                  <div key={lyr.num} className="nb-cell flex flex-col gap-2">
                    <div className="flex justify-between items-baseline">
                      <span className="font-mono text-[0.6rem] text-steel uppercase tracking-[0.05em]">
                        {lyr.tag}
                      </span>
                      <span className="font-mono text-[0.6rem] text-amber font-bold">
                        LAYER {lyr.num}
                      </span>
                    </div>
                    <h3 className="font-display text-xl font-bold mt-2 mb-0 text-foreground">
                      {lyr.title}
                    </h3>
                    <p className="text-[0.82rem] text-muted leading-relaxed m-0">{lyr.body}</p>
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
                <div className="nb-label">02 / 03 — Profiling</div>
                <h2 className="font-display text-[clamp(2rem,4vw,3rem)] font-extrabold tracking-[-0.04em] my-5 leading-tight">
                  Query Latency
                </h2>
                <p className="font-sans text-[0.95rem] text-muted leading-relaxed mb-0">
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
                <div className="nb-label">03 / 03 — Specifications</div>
                <h2 className="font-display text-[clamp(2rem,4vw,3rem)] font-extrabold tracking-[-0.04em] my-5 leading-tight">
                  Operational Limits
                </h2>
                <p className="font-sans text-[0.95rem] text-muted leading-relaxed mb-0">
                  Technical limits enforced at memory layer boundaries to prevent out-of-memory
                  states during heavy concurrent query evaluations.
                </p>
              </div>
              <div>
                <div className="nb-frame" data-frame-label="SPECS">
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
              <div className="flex items-center justify-between gap-4 flex-wrap">
                <div>
                  <div className="nb-label" style={{ color: "var(--text-on-amber)" }}>
                    GET STARTED
                  </div>
                  <h2
                    className="font-display text-2xl font-extrabold"
                    style={{ color: "var(--text-on-amber)" }}
                  >
                    Compiles to native. Runs in your process.
                  </h2>
                  <p className="text-sm" style={{ color: "var(--text-on-amber)", opacity: 0.8 }}>
                    Install VantaDB in one command.
                  </p>
                </div>
                <code
                  className="font-mono text-lg font-bold"
                  style={{ color: "var(--text-on-amber)" }}
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

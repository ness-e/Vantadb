import { createLazyRoute } from "@tanstack/react-router";
import { useRef, useState } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
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
            className={`architecture-profiler-segment ${seg.id === hoveredSegment ? "architecture-profiler-segment--hovered" : ""}`}
            style={{ width: `${seg.share}%` }}
            onMouseEnter={() => setHoveredSegment(seg.id)}
            onMouseLeave={() => setHoveredSegment(null)}
          >
            <span
              className={`architecture-profiler-pct ${seg.share > 15 ? "architecture-profiler-pct--visible" : "architecture-profiler-pct--hidden"}`}
              style={{ color: seg.id === hoveredSegment ? "#000000" : "var(--background)" }}
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
              {hoveredData.label} \u2014 {hoveredData.share}% of query budget
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
      tag: "Python & Rust bindings",
      title: "Zero-Copy SDK",
      body: "High-level importable libraries. Wraps raw Rust structures via PyO3, translating namespaces, parameters, and query lists without GIL overhead.",
    },
    {
      tag: "Compiles to native library",
      title: "Stable FFI Boundary",
      body: "A clean FFI contract with zero IPC, sockets, or network overhead. Memory pointers are shared directly between Python interpreter memory and Rust heap.",
    },
    {
      tag: "Multi-modal planning",
      title: "Core Search Optimizer",
      body: "Analyzes filtering conditions and queries. Directs traffic to keyword databases and HNSW indexing systems, blending results through fast RRF fusers.",
    },
    {
      tag: "Log-Structured persist",
      title: "Storage Engine",
      body: "Coordinates active memory buffers, Write-Ahead Logs, block sync thresholds, and backgrounds compaction passes directly into a single database file.",
    },
  ];

  const stackRef = useRef<HTMLElement>(null);
  const profilerRef = useRef<HTMLElement>(null);
  const limitsRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(stackRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, stackRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(profilerRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, profilerRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(limitsRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, limitsRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p05"
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
        <NbSection ref={stackRef} ariaLabel="Stack layers">
          <NbSectionHeader
            monoLabel="[STACK LAYERS]"
            headline="From SDK to storage."
            sub="VantaDB provides safe bindings on top of a highly optimized multi-modal execution core and storage layer."
          />

          <div className="nb-grid nb-grid--cols-2">
            {layers.map((lyr, i) => (
              <div key={i} className="nb-cell architecture-layer-cell nb-engine-part">
                <div className="architecture-layer-header">
                  <span className="architecture-layer-tag">{lyr.tag}</span>
                  <span className="architecture-layer-num">LAYER 0{i + 1}</span>
                </div>
                <h3 className="architecture-layer-title">{lyr.title}</h3>
                <p className="architecture-layer-body">{lyr.body}</p>
              </div>
            ))}
          </div>
        </NbSection>

        <div className="nb-divider" />

        <NbSection ref={profilerRef} ariaLabel="Query latency profiler">
          <NbSectionHeader
            monoLabel="[QUERY LATENCY]"
            headline="12% overhead. 88% search."
            sub="Due to direct sharing of pointer addresses, the cost of crossing FFI bindings is less than 12% of total search time, leaving CPU resources free to evaluate similarity indexes."
          />
          <div className="nb-engine-part">
            <PerformanceProfiler />
          </div>
        </NbSection>

        <div className="nb-divider" />

        <NbSection ref={limitsRef} ariaLabel="Operational limits">
          <NbSectionHeader
            monoLabel="[OPERATIONAL LIMITS]"
            headline="Boundaries you can count on."
            sub="Technical limits enforced at memory layer boundaries to prevent out-of-memory states during heavy concurrent query evaluations."
          />

          <div className="nb-engine-part">
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
                    label="Quantization Methods"
                    val="SQ8 · TurboQuant · RaBitQ"
                    desc="8-bit scalar, 3-bit turbo, or 1-bit RaBitQ — 4× to 32× memory reduction over full f32 vectors."
                  />
                  <SpecRow
                    label="Graph Traversal"
                    val="BFS · DFS · Topo · DAG"
                    desc="Built-in BFS/DFS traversal, topological sort, and DAG cycle detection over directed adjacency edges."
                  />
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
                    val="1 Writer \u00B7 \u221E Readers"
                    desc="Safe thread concurrent read executions, write paths synchronize locks to protect journal logs."
                  />
                  <SpecRow
                    label="sync_mode Options"
                    val="always | periodic | never"
                    desc="Set flush WAL constraints depending on durability trade-offs (Periodic sync defaults to 500ms intervals)."
                  />
                  <SpecRow
                    label="Engine States"
                    val="Init \u2192 Ready \u2192 Flush \u2192 Closed"
                    desc="Exposes state transitions cleanly through FFI to allow hot rebuilding without data losses."
                  />
                  <SpecRow
                    label="Memory Prefetching"
                    val="madvise / PrefetchVirtualMemory"
                    desc="Predictive kernel prefetching on Unix (madvise) and Windows (PrefetchVirtualMemory) for cold vector pages."
                  />
                  <SpecRow
                    label="Data Expiry"
                    val="TTL auto-eviction"
                    desc="Time-to-live based automatic record expiry with background compaction."
                  />
                  <SpecRow
                    label="Batch Operations"
                    val="Rayon parallelism"
                    desc="Batch put/get/delete operations parallelized across all available CPU cores."
                  />
                </tbody>
              </table>
            </div>
          </div>
        </NbSection>

        <NbSection variant="dark" className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="architecture-cta-wrap">
              <div>
                <h2 className="architecture-cta-title">
                  Compiles to native. Runs in your process.
                </h2>
                <p className="architecture-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="architecture-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}

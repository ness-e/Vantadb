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

const LAYERS = [
  {
    tag: "Python & Rust bindings",
    title: "Zero-Copy SDK",
    desc: "High-level importable libraries. Wraps raw Rust structures via PyO3, translating namespaces, parameters, and query lists without GIL overhead.",
    metric: "~1\u00B5s overhead",
  },
  {
    tag: "Compiles to native library",
    title: "Stable FFI Boundary",
    desc: "A clean FFI contract with zero IPC, sockets, or network overhead. Memory pointers are shared directly between Python interpreter memory and Rust heap.",
    metric: "12% of query budget",
  },
  {
    tag: "Multi-modal planning",
    title: "Core Search Optimizer",
    desc: "Analyzes filtering conditions and queries. Directs traffic to keyword databases and HNSW indexing systems, blending results through fast RRF fusers.",
    metric: "48% of query time",
  },
  {
    tag: "Log-Structured persist",
    title: "Storage Engine",
    desc: "Coordinates active memory buffers, Write-Ahead Logs, block sync thresholds, and backgrounds compaction passes directly into a single database file.",
    metric: "16% commit cost",
  },
];

const SPECS = [
  {
    label: "Quantization Methods",
    val: "SQ8 \u00B7 TurboQuant \u00B7 RaBitQ",
    desc: "8-bit scalar, 3-bit turbo, or 1-bit RaBitQ \u2014 4\u00D7 to 32\u00D7 memory reduction over full f32 vectors.",
    amber: false,
  },
  {
    label: "Graph Traversal",
    val: "BFS \u00B7 DFS \u00B7 Topo \u00B7 DAG",
    desc: "Built-in BFS/DFS traversal, topological sort, and DAG cycle detection over directed adjacency edges.",
    amber: false,
  },
  {
    label: "Key Size Limit",
    val: "1 KB",
    desc: "Identifiers must be compact to ensure lightning-fast pointer indexing in standard LSM memory bounds.",
    amber: false,
  },
  {
    label: "Vector Dimensions",
    val: "32,768 dims",
    desc: "Configurable HNSW index structures, optimized up to high-end sparse embeddings.",
    amber: true,
  },
  {
    label: "Text Content Limit",
    val: "10 MB per put",
    desc: "Larger chunk payloads are offloaded directly to binary buffers, preserving planner heap latency floors.",
    amber: false,
  },
  {
    label: "Metadata Size",
    val: "64 KB",
    desc: "Structured dictionaries for lexical pre-filtering operations before RRF fusion runs.",
    amber: false,
  },
  {
    label: "Concurrency Model",
    val: "1 Writer \u00B7 \u221E Readers",
    desc: "Safe thread concurrent read executions, write paths synchronize locks to protect journal logs.",
    amber: false,
  },
  {
    label: "sync_mode",
    val: "always | periodic | never",
    desc: "Set flush WAL constraints depending on durability trade-offs. Periodic sync defaults to 500ms intervals.",
    amber: false,
  },
  {
    label: "Engine States",
    val: "Init \u2192 Ready \u2192 Flush \u2192 Closed",
    desc: "Exposes state transitions cleanly through FFI to allow hot rebuilding without data losses.",
    amber: false,
  },
  {
    label: "Memory Prefetching",
    val: "madvise / PrefetchVirtualMemory",
    desc: "Predictive kernel prefetching on Unix (madvise) and Windows (PrefetchVirtualMemory) for cold vector pages.",
    amber: true,
  },
  {
    label: "Data Expiry",
    val: "TTL auto-eviction",
    desc: "Time-to-live based automatic record expiry with background compaction.",
    amber: false,
  },
  {
    label: "Batch Operations",
    val: "Rayon parallelism",
    desc: "Batch put/get/delete operations parallelized across all available CPU cores.",
    amber: false,
  },
];

const SEGMENTS = [
  {
    id: "ffi",
    label: "FFI Boundary Translation",
    share: 12,
    desc: "Fast PyO3 compilation translating Python dictionaries to native Rust structs with zero memory copies.",
    bg: "var(--steel)",
  },
  {
    id: "planner",
    label: "Query Optimization",
    share: 24,
    desc: "Sifts through filters, sets up lexical scanning ranges, and prepares candidate HNSW entry points.",
    bg: "var(--muted)",
  },
  {
    id: "index",
    label: "HNSW & BM25 Traversal",
    share: 48,
    desc: "Core path searching approximate nearest neighbors and evaluating index statistics in parallel.",
    bg: "var(--amber)",
  },
  {
    id: "wal",
    label: "WAL Commit & fsync",
    share: 16,
    desc: "Forces durability logging, computes transaction CRC32C, and syncs sectors on disks.",
    bg: "var(--foreground)",
  },
];

function FlowLayer({
  layer,
  i,
  pos,
}: {
  layer: (typeof LAYERS)[number];
  i: number;
  pos: "top" | "btm";
}) {
  return (
    <div className={`arch-flow-card arch-flow-card--${pos} nb-engine-part`}>
      <div className="arch-flow-card-header">
        <span className="arch-flow-tag">{layer.tag}</span>
        <span className="arch-flow-badge">LAYER 0{i + 1}</span>
      </div>
      <h3 className="arch-flow-title">{layer.title}</h3>
      <p className="arch-flow-desc">{layer.desc}</p>
      <span className="arch-flow-metric">{layer.metric}</span>
    </div>
  );
}

function PerformanceProfiler() {
  const [hoveredSegment, setHoveredSegment] = useState<string | null>(null);

  const hoveredData = SEGMENTS.find((s) => s.id === hoveredSegment);

  return (
    <div className="arch-profiler nb-engine-part">
      <div className="arch-profiler-header">
        <span className="arch-profiler-title">CPU TIME DISTRIBUTION</span>
        <span className="arch-profiler-hint">HOVER SEGMENTS TO PROFILE</span>
      </div>

      <div className="arch-profiler-bar">
        {SEGMENTS.map((seg) => (
          <div
            key={seg.id}
            className="arch-profiler-segment"
            style={{
              width: `${seg.share}%`,
              background: seg.id === hoveredSegment ? "var(--amber)" : seg.bg,
            }}
            onMouseEnter={() => setHoveredSegment(seg.id)}
            onMouseLeave={() => setHoveredSegment(null)}
          >
            <span
              className={`arch-profiler-pct ${seg.share > 15 ? "arch-profiler-pct--visible" : "arch-profiler-pct--hidden"}`}
              style={{ color: seg.id === hoveredSegment ? "#000" : "var(--background)" }}
            >
              {seg.share}%
            </span>
          </div>
        ))}
      </div>

      <div className="arch-profiler-detail">
        {hoveredData ? (
          <div>
            <div className="arch-profiler-detail-label">
              {hoveredData.label} &mdash; {hoveredData.share}% of query budget
            </div>
            <p className="arch-profiler-detail-desc">{hoveredData.desc}</p>
          </div>
        ) : (
          <div>
            <div className="arch-profiler-detail-label arch-profiler-detail-label--muted">
              Engine Performance Summary
            </div>
            <p className="arch-profiler-detail-desc">
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
  const flowRef = useRef<HTMLElement>(null);
  const profilerRef = useRef<HTMLElement>(null);
  const specsRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(flowRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, flowRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(profilerRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, profilerRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(specsRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, specsRef);

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
        <NbSection ref={flowRef} ariaLabel="Architecture pipeline">
          <NbSectionHeader
            monoLabel="[ARCHITECTURE]"
            headline="How data flows."
            sub="From Python SDK to disk \u2014 four layers, zero copies, one process."
          />

          <div className="arch-flow">
            <FlowLayer layer={LAYERS[0]} i={0} pos="top" />
            <div className="arch-flow-arrow-r" aria-hidden="true" />
            <FlowLayer layer={LAYERS[1]} i={1} pos="top" />
            <div className="arch-flow-arrow-btm" aria-hidden="true" />
            <FlowLayer layer={LAYERS[2]} i={2} pos="btm" />
            <div className="arch-flow-arrow-r" aria-hidden="true" />
            <FlowLayer layer={LAYERS[3]} i={3} pos="btm" />
          </div>
        </NbSection>

        <div className="nb-divider" />

        <NbSection ref={profilerRef} ariaLabel="Query latency profiler">
          <NbSectionHeader
            monoLabel="[QUERY LATENCY]"
            headline="12% overhead. 88% search."
            sub="Due to direct sharing of pointer addresses, the cost of crossing FFI bindings is less than 12% of total search time, leaving CPU resources free to evaluate similarity indexes."
          />

          <PerformanceProfiler />
        </NbSection>

        <div className="nb-divider" />

        <NbSection ref={specsRef} ariaLabel="Operational limits">
          <NbSectionHeader
            monoLabel="[OPERATIONAL LIMITS]"
            headline="Boundaries you can count on."
            sub="Technical limits enforced at memory layer boundaries to prevent out-of-memory states during heavy concurrent query evaluations."
          />

          <div className="arch-specs nb-engine-part">
            {SPECS.map((spec, i) => (
              <div key={i} className="arch-spec-card">
                <span className="arch-spec-label">{spec.label}</span>
                <span className={`arch-spec-val ${spec.amber ? "arch-spec-val--amber" : ""}`}>
                  {spec.val}
                </span>
                <p className="arch-spec-desc">{spec.desc}</p>
              </div>
            ))}
          </div>
        </NbSection>

        <NbSection variant="dark" className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="arch-cta-wrap">
              <div>
                <h2 className="arch-cta-title">
                  Compiles to native. Runs in your process.
                </h2>
                <p className="arch-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="arch-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}

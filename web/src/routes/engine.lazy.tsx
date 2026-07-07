import { createLazyRoute } from "@tanstack/react-router";
import { useRef, useState, useEffect } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/engine.css";

export const Route = createLazyRoute("/engine")({
  component: EnginePage,
  pendingComponent: PendingComponent,
});

const GRAPH_NODES = [
  { x: 160, y: 90, label: "agent:main", size: 14 },
  { x: 320, y: 50, label: "memory:001", size: 10 },
  { x: 280, y: 180, label: "context:rag", size: 11 },
  { x: 80, y: 200, label: "vector:embed", size: 9 },
  { x: 420, y: 130, label: "hnsw:idx", size: 10 },
  { x: 370, y: 240, label: "edge:weight", size: 8 },
  { x: 150, y: 290, label: "bm25:score", size: 9 },
  { x: 460, y: 60, label: "namespace:db", size: 12 },
];

const GRAPH_EDGES: [number, number][] = [
  [0, 1], [0, 3], [0, 2], [1, 4],
  [2, 4], [2, 5], [3, 6], [4, 7],
  [1, 7], [2, 6],
];

const PIPELINE_STAGES = [
  { name: "Query", desc: "Tokenizer", accent: "amber" },
  { name: "BM25", desc: "Lexical Score", accent: "steel" },
  { name: "HNSW", desc: "Vector Recall", accent: "steel" },
  { name: "RRF", desc: "Fused Ranker", accent: "amber" },
  { name: "Edges", desc: "Local Graph", accent: "steel" },
  { name: "WAL", desc: "Durable Write", accent: "steel" },
] as const;

function GraphTopology() {
  const [activeNode, setActiveNode] = useState<number | null>(null);

  const isEdgeHighlighted = (a: number, b: number) => {
    if (activeNode === null) return false;
    return a === activeNode || b === activeNode;
  };

  return (
    <svg
      viewBox="0 0 540 330"
      className="engine-svg-graph"
      aria-label="Graph database node connection visualization"
    >
      <defs>
        <radialGradient id="ng" r="50%">
          <stop offset="0%" stopColor="var(--amber)" stopOpacity="0.3" />
          <stop offset="100%" stopColor="var(--amber)" stopOpacity="0" />
        </radialGradient>
      </defs>
      {GRAPH_EDGES.map(([a, b]) => (
        <line
          key={`${a}-${b}`}
          x1={GRAPH_NODES[a].x}
          y1={GRAPH_NODES[a].y}
          x2={GRAPH_NODES[b].x}
          y2={GRAPH_NODES[b].y}
          stroke={isEdgeHighlighted(a, b) ? "var(--amber)" : "var(--border)"}
          strokeWidth={isEdgeHighlighted(a, b) ? "1.5" : "1"}
          strokeDasharray={isEdgeHighlighted(a, b) ? "none" : "2 2"}
          className="engine-graph-edge"
        />
      ))}
      {GRAPH_NODES.map((n, i) => (
        <g
          key={n.label}
          className="engine-graph-node"
          onMouseEnter={() => setActiveNode(i)}
          onMouseLeave={() => setActiveNode(null)}
        >
          {activeNode === i && <circle cx={n.x} cy={n.y} r={n.size * 3} fill="url(#ng)" />}
          <circle
            cx={n.x}
            cy={n.y}
            r={n.size}
            fill="var(--surface)"
            stroke={activeNode === i ? "var(--amber)" : "var(--border)"}
            strokeWidth="1"
            className="engine-graph-ring"
          />
          <circle
            cx={n.x}
            cy={n.y}
            r="3"
            fill={activeNode === i ? "var(--amber)" : "var(--steel)"}
          />
          <text
            x={n.x}
            y={n.y - n.size - 5}
            textAnchor="middle"
            fill={activeNode === i ? "var(--foreground)" : "var(--muted)"}
            fontSize="9"
            fontFamily="var(--font-mono)"
            fontWeight="600"
            className="engine-graph-label"
          >
            {n.label}
          </text>
        </g>
      ))}
    </svg>
  );
}

function RRFWeightsSlider() {
  const [bm25Weight, setBm25Weight] = useState(50);
  const hnswWeight = 100 - bm25Weight;

  const lexicalRecall = Math.round(bm25Weight * 0.7 + 10);
  const vectorRecall = Math.round(hnswWeight * 0.8 + 15);
  const fusedRecall = Math.round(100 - Math.abs(bm25Weight - 45) * 0.15);
  const queryLatency = (1.2 + (hnswWeight / 100) * 0.4).toFixed(2);

  return (
    <div className="nc-engine-slider-wrap">
      <div className="nc-engine-slider-header">
        <span className="nc-engine-slider-title">RRF Weights Planner</span>
        <span className="nc-engine-slider-latency">LATENCY: {queryLatency}ms</span>
      </div>

      <p className="nc-engine-slider-desc">
        Adjust the slider to coordinate keyword recall against vector space clustering.
      </p>

      <div className="nc-engine-slider-labels">
        <span>BM25: {bm25Weight}%</span>
        <span>HNSW: {hnswWeight}%</span>
      </div>

      <input
        type="range"
        min="0"
        max="100"
        value={bm25Weight}
        onChange={(e) => setBm25Weight(Number(e.target.value))}
        className="nc-engine-slider-input"
        aria-label="BM25 to HNSW fusion weight ratio"
      />

      <div className="nc-engine-slider-stats">
        <div className="nc-engine-slider-stat">
          <span className="nc-engine-slider-stat-label">LEXICAL RECALL</span>
          <span className="nc-engine-slider-stat-value">{lexicalRecall}%</span>
        </div>
        <div className="nc-engine-slider-stat">
          <span className="nc-engine-slider-stat-label">VECTOR RECALL</span>
          <span className="nc-engine-slider-stat-value">{vectorRecall}%</span>
        </div>
        <div className="nc-engine-slider-stat nc-engine-slider-stat--fused">
          <span className="nc-engine-slider-stat-label nc-engine-slider-stat-label--amber">FUSED @10</span>
          <span className="nc-engine-slider-stat-value nc-engine-slider-stat-value--amber">{fusedRecall}%</span>
        </div>
      </div>
    </div>
  );
}

function WALSimulator() {
  const [logs, setLogs] = useState<string[]>([
    "[00:00:01] System boot initialized",
    "[00:00:02] Storage engine opened at path ./agent_memory",
    "[00:00:03] Replaying WAL logs... 0 transactions found",
    "[00:00:04] Database state: READY",
  ]);
  const [engineState, setEngineState] = useState<"ready" | "crashed" | "recovering">("ready");
  const consoleEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (consoleEndRef.current) {
      consoleEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs]);

  const triggerCrash = () => {
    if (engineState !== "ready") return;
    setEngineState("crashed");
    setLogs((prev) => [
      ...prev,
      "[00:02:15] put txn: namespace=memories, key=conv-88",
      "[00:02:16] wal: writing page log CRC32C=0xab12de",
      "!!! CRITICAL FAILURE: PROCESS TERMINATED OUTSIDE CLEAN DISCONNECT !!!",
      "STATUS: OFFLINE",
    ]);
  };

  const recoverFromWAL = () => {
    if (engineState !== "crashed") return;
    setEngineState("recovering");
    setLogs((prev) => [
      ...prev,
      "[00:03:01] Database reopened. Initializing WAL scan...",
      "[00:03:02] WAL found. Unflushed write at sector index 43",
      "[00:03:03] Checking integrity: verifying CRC32C checksums...",
      "[00:03:04] Checksum 0xab12de OK. Syncing WAL entry 1/1",
      "[00:03:05] WAL sync finished. Rebuilding transient HNSW indexes",
      "[00:03:06] State restored in 0.4ms. 1 transaction recovered.",
      "STATUS: READY",
    ]);
    setTimeout(() => {
      setEngineState("ready");
    }, 1200);
  };

  const getLogLevel = (log: string) => {
    if (log.includes("!!!")) return "error";
    if (log.includes("READY") || log.includes("OK")) return "ok";
    if (log.includes("RECOVER") || log.includes("Syncing")) return "warn";
    return "info";
  };

  return (
    <div className="nc-engine-recorder">
      <div className="nc-engine-recorder-header">
        <div className="nc-engine-recorder-status">
          <span className="nc-engine-recorder-dot" data-state={engineState} />
          <span className="nc-engine-recorder-status-text">STATUS: {engineState.toUpperCase()}</span>
        </div>
        <div className="nc-engine-recorder-actions">
          <button
            className="nb-btn nb-btn--ghost"
            onClick={triggerCrash}
            disabled={engineState !== "ready"}
          >
            CRASH ENGINE
          </button>
          <button className="nb-btn" onClick={recoverFromWAL} disabled={engineState !== "crashed"}>
            RECOVER FROM WAL
          </button>
        </div>
      </div>

      <div className="nc-engine-recorder-console">
        {logs.map((log) => (
          <div key={log} className="nc-engine-recorder-line" data-level={getLogLevel(log)}>
            {log}
          </div>
        ))}
        <div ref={consoleEndRef} />
      </div>
    </div>
  );
}

function ArchitecturePipeline() {
  return (
    <div className="nc-engine-conveyor">
      <div className="nc-engine-conveyor-track">
        {PIPELINE_STAGES.map((s, i) => (
          <div key={s.name} className="nc-engine-conveyor-stage">
            <div className="nc-engine-conveyor-card" data-accent={s.accent}>
              <div className="nc-engine-conveyor-card-name">{s.name}</div>
              <div className="nc-engine-conveyor-card-desc">{s.desc}</div>
            </div>
            {i < PIPELINE_STAGES.length - 1 && (
              <span className="nc-engine-conveyor-arrow" aria-hidden="true">→</span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

function EnginePage() {
  const hybridRef = useRef<HTMLElement>(null);
  const graphRef = useRef<HTMLElement>(null);
  const walRef = useRef<HTMLElement>(null);
  const pipelineRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({
      scrollTrigger: scrollTriggerConfig(hybridRef.current, 60),
    });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, hybridRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({
      scrollTrigger: scrollTriggerConfig(graphRef.current, 60),
    });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, graphRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({
      scrollTrigger: scrollTriggerConfig(walRef.current, 60),
    });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, walRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({
      scrollTrigger: scrollTriggerConfig(pipelineRef.current, 60),
    });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, pipelineRef);

  return (
    <div className="nc-engine-crt">
      <NbSubpageHero
        pattern="p01"
        title={
          <span>
            Four modalities.
            <br />
            One atomic contract.
          </span>
        }
        sub="VantaDB consolidates lexical matching, HNSW vector search, local graph edges and transactional persistence in a zero-dependency Rust local-first library database."
      />

      <main>
        <NbSection ref={hybridRef} variant="lg" ariaLabel="Hybrid search">
          <NbSectionHeader
            monoLabel="[HYBRID SEARCH]"
            headline="BM25 + HNSW + RRF."
            sub="VantaDB query planner optimizes combined metadata filters, HNSW vector similarity, and BM25 full-text queries, synthesizing them into a single-pass execution plan."
          />

          <div className="nc-engine-panel nc-engine-section nc-engine-part">
            <div className="nc-engine-panel-label">Instrument: Hybrid Fusion</div>
            <p className="nc-engine-slider-desc">
              Each query pass is fused through Reciprocal Rank Fusion, giving you the precision of
              keyword search with the semantic reach of vector embeddings — without managing
              separate infrastructure.
            </p>

            <div className="nb-grid nb-grid--cols-2">
              <div className="nc-engine-gauge">
                <div className="nc-engine-gauge-value">~1.2ms</div>
                <span className="nc-engine-gauge-label">P50 Latency</span>
                <span className="nc-engine-gauge-unit">Lexical BM25 search</span>
              </div>
              <div className="nc-engine-gauge">
                <div className="nc-engine-gauge-value">0.998</div>
                <span className="nc-engine-gauge-label">Recall@10</span>
                <span className="nc-engine-gauge-unit">Full-text recall rate</span>
              </div>
              <div className="nc-engine-gauge">
                <div className="nc-engine-gauge-value">M=16</div>
                <span className="nc-engine-gauge-label">HNSW Connections</span>
                <span className="nc-engine-gauge-unit">Graph density</span>
              </div>
              <div className="nc-engine-gauge">
                <div className="nc-engine-gauge-value">SQ8</div>
                <span className="nc-engine-gauge-label">Quantization</span>
                <span className="nc-engine-gauge-unit">Memory compression</span>
              </div>
            </div>
          </div>
        </NbSection>

        <NbSection ref={graphRef} ariaLabel="Graph topology">
          <NbSectionHeader
            monoLabel="[GRAPH QUERY]"
            headline="Knowledge topology."
            sub="Hover nodes to explore in-memory relations. VantaDB stores directed adjacency lists alongside vectors — supporting BFS, DFS, topological sort, and DAG cycle detection for graph-based agent memory."
          />

          <div className="nb-grid nb-grid--cols-2 nc-engine-section">
            <div className="nc-engine-panel nc-engine-part nc-engine-radar">
              <div className="nc-engine-panel-label">Live Topology · Hover to traverse</div>
              <GraphTopology />
            </div>
            <div className="nc-engine-part">
              <RRFWeightsSlider />
            </div>
          </div>
        </NbSection>

        <NbSection ref={walRef} ariaLabel="WAL durability">
          <NbSectionHeader
            monoLabel="[DURABILITY]"
            headline="Crash-safe WAL."
            sub="VantaDB guarantees complete transaction safety. Write-Ahead Logging forces log flushes before write acknowledgment, recovering state instantly on reboot."
          />

          <div className="nc-engine-section nc-engine-part">
            <p className="nc-engine-slider-desc">
              The WAL journal uses CRC32C integrity checksums with fsync-on-write semantics. On
              crash, automatic log replay detects the last consistent checkpoint and restores
              state in under 1ms.
            </p>
            <WALSimulator />
          </div>
        </NbSection>

        <NbSection ref={pipelineRef} ariaLabel="Query pipeline">
          <NbSectionHeader
            monoLabel="[PIPELINE]"
            headline="End-to-end query execution."
            sub="A query travels through six stages — from parsing and hybrid search through graph traversal to a durable write confirmation."
          />

          <div className="nc-engine-section nc-engine-part">
            <ArchitecturePipeline />
          </div>
        </NbSection>

        <NbSection variant="dark" className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="nc-engine-cta">
              <div>
                <h2 className="nc-engine-cta-heading">Four modalities. One dependency.</h2>
                <p className="nc-engine-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="nc-engine-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}

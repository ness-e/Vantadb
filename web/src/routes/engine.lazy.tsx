import { createLazyRoute } from "@tanstack/react-router";
import { useEffect, useState, useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/engine")({
  component: EnginePage,
  pendingComponent: PendingComponent,
});

function GraphTopology() {
  const [activeNode, setActiveNode] = useState<number | null>(null);
  const nodes = [
    { x: 160, y: 90, label: "agent:main", size: 14 },
    { x: 320, y: 50, label: "memory:001", size: 10 },
    { x: 280, y: 180, label: "context:rag", size: 11 },
    { x: 80, y: 200, label: "vector:embed", size: 9 },
    { x: 420, y: 130, label: "hnsw:idx", size: 10 },
    { x: 370, y: 240, label: "edge:weight", size: 8 },
    { x: 150, y: 290, label: "bm25:score", size: 9 },
    { x: 460, y: 60, label: "namespace:db", size: 12 },
  ];
  const edges = [
    [0, 1],
    [0, 3],
    [0, 2],
    [1, 4],
    [2, 4],
    [2, 5],
    [3, 6],
    [4, 7],
    [1, 7],
    [2, 6],
  ];

  const isEdgeHighlighted = (a: number, b: number) => {
    if (activeNode === null) return false;
    return a === activeNode || b === activeNode;
  };

  return (
    <svg
      viewBox="0 0 540 330"
      className="graph-topology-svg"
      style={{ display: "block", background: "var(--background)" }}
      aria-label="Graph database node connection visualization"
    >
      <defs>
        <radialGradient id="ng" r="50%">
          <stop offset="0%" stopColor="var(--amber)" stopOpacity="0.3" />
          <stop offset="100%" stopColor="var(--amber)" stopOpacity="0" />
        </radialGradient>
      </defs>
      {edges.map(([a, b]) => (
        <line
          key={`${a}-${b}`}
          x1={nodes[a].x}
          y1={nodes[a].y}
          x2={nodes[b].x}
          y2={nodes[b].y}
          stroke={isEdgeHighlighted(a, b) ? "var(--amber)" : "var(--border)"}
          strokeWidth={isEdgeHighlighted(a, b) ? "1.5" : "1"}
          strokeDasharray={isEdgeHighlighted(a, b) ? "none" : "2 2"}
          style={{ transition: "stroke 150ms ease, stroke-width 150ms ease" }}
        />
      ))}
      {nodes.map((n, i) => (
        <g
          key={n.label}
          style={{ cursor: "pointer" }}
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
            style={{ transition: "stroke 150ms ease" }}
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
            style={{ transition: "fill 150ms ease" }}
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
    <div>
      <div className="flex items-baseline justify-between border-b border-[var(--border-visible)] pb-4 mb-6">
        <h4 className="font-mono text-[0.8rem] uppercase tracking-[0.08em] m-0 font-semibold">
          RRF Weights Planner
        </h4>
        <span className="font-mono text-[0.7rem]">LATENCY: {queryLatency}ms</span>
      </div>

      <p className="text-[0.85rem] text-muted leading-relaxed m-0 mb-6">
        Adjust the slider to coordinate keyword recall against vector space clustering.
      </p>

      <div className="flex justify-between font-mono text-[0.7rem] font-semibold mb-4">
        <span>BM25: {bm25Weight}%</span>
        <span>HNSW: {hnswWeight}%</span>
      </div>

      <input
        type="range"
        min="0"
        max="100"
        value={bm25Weight}
        onChange={(e) => setBm25Weight(Number(e.target.value))}
        className="rrf-slider w-full my-6"
        aria-label="BM25 to HNSW fusion weight ratio"
      />

      <div className="nb-grid nb-grid--cols-3">
        <div className="nb-cell">
          <div className="font-mono text-[0.55rem] text-steel uppercase tracking-[0.05em]">
            LEXICAL RECALL
          </div>
          <div className="font-mono text-[1.1rem] font-bold text-foreground">{lexicalRecall}%</div>
        </div>
        <div className="nb-cell">
          <div className="font-mono text-[0.55rem] text-steel uppercase tracking-[0.05em]">
            VECTOR RECALL
          </div>
          <div className="font-mono text-[1.1rem] font-bold text-foreground">{vectorRecall}%</div>
        </div>
        <div className="nb-cell" style={{ borderLeft: "2px solid var(--amber)" }}>
          <div className="font-mono text-[0.55rem] text-amber font-bold uppercase tracking-[0.05em]">
            FUSED @10
          </div>
          <div className="font-mono text-[1.1rem] font-bold text-amber">{fusedRecall}%</div>
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

  const getLogColor = (log: string) => {
    if (log.includes("!!!")) return "var(--danger)";
    if (log.includes("READY") || log.includes("OK")) return "var(--amber)";
    if (log.includes("RECOVER") || log.includes("Syncing")) return "var(--steel)";
    return "var(--muted)";
  };

  return (
    <div>
      <div className="flex items-center justify-between border-b border-[var(--border-visible)] pb-4 mb-6">
        <div className="flex items-center gap-3">
          <span
            className="inline-block"
            style={{
              width: 8,
              height: 8,
              background:
                engineState === "ready"
                  ? "var(--amber)"
                  : engineState === "crashed"
                    ? "var(--danger)"
                    : "var(--steel)",
            }}
          />
          <span className="font-mono text-[0.72rem]">STATUS: {engineState}</span>
        </div>
        <div className="flex gap-2">
          <button
            className="nb-btn nb-btn--ghost px-3 py-[0.4rem] text-[0.65rem] font-mono uppercase"
            onClick={triggerCrash}
            disabled={engineState !== "ready"}
          >
            CRASH ENGINE
          </button>
          <button
            className="nb-btn px-3 py-[0.4rem] text-[0.65rem] font-mono uppercase"
            onClick={recoverFromWAL}
            disabled={engineState !== "crashed"}
          >
            RECOVER FROM WAL
          </button>
        </div>
      </div>

      <div
        className="wal-console font-mono text-[0.7rem] h-[180px] overflow-y-auto leading-relaxed border-2 border-[var(--border-visible)] p-5"
        style={{ background: "var(--surface)" }}
      >
        {logs.map((log) => (
          <div
            key={log}
            className="py-1 border-b border-[var(--border)]"
            style={{ color: getLogColor(log) }}
          >
            {log}
          </div>
        ))}
        <div ref={consoleEndRef} />
      </div>
    </div>
  );
}

function ArchitecturePipeline() {
  const stages = [
    { name: "Query", desc: "Tokenizer", color: "var(--amber)" },
    { name: "BM25", desc: "Lexical Score", color: "var(--steel)" },
    { name: "HNSW", desc: "Vector Recall", color: "var(--steel)" },
    { name: "RRF", desc: "Fused Ranker", color: "var(--amber)" },
    { name: "Edges", desc: "Local Graph", color: "var(--steel)" },
    { name: "WAL", desc: "Durable Write", color: "var(--steel)" },
  ];

  return (
    <section className="nb-section">
      <div className="nb-inner">
        <div className="nb-label">03 / 03 — Pipeline</div>
        <h2 className="font-display text-[clamp(2rem,4vw,3rem)] font-extrabold tracking-[-0.04em] my-5 mb-12 leading-tight">
          End-to-End Query Execution
        </h2>

        <div
          className="overflow-x-auto border-2 border-[var(--border-visible)] p-8 py-12"
          style={{ background: "var(--surface)" }}
        >
          <div className="flex gap-4 items-center min-w-max">
            {stages.map((s, i) => (
              <div key={s.name} className="flex items-center gap-4">
                <div
                  className="min-w-[140px] text-left"
                  style={{
                    border: `2px solid ${s.color}`,
                    padding: "1.25rem 2rem",
                    background: "var(--background)",
                  }}
                >
                  <div className="font-display text-[0.9rem] font-bold text-foreground">
                    {s.name}
                  </div>
                  <div className="font-mono text-[0.55rem] text-muted uppercase mt-1">{s.desc}</div>
                </div>
                {i < stages.length - 1 && (
                  <span className="font-mono text-[var(--border-visible)] text-base">→</span>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

function EnginePage() {
  return (
    <div>
      <NbSubpageHero
        num="01"
        eyebrow="Core Engine"
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
        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-asymmetric">
              <div>
                <div className="nb-label">01 / 03 — Hybrid Search</div>
                <h2 className="font-display text-[clamp(2rem,4vw,3rem)] font-extrabold tracking-[-0.04em] my-5 leading-tight">
                  BM25 + HNSW + RRF
                </h2>
                <p className="font-sans text-[0.95rem] text-muted leading-relaxed">
                  VantaDB query planner optimizes combined metadata filters, HNSW vector similarity,
                  and BM25 full-text queries, synthesizing them into a single-pass execution plan.
                </p>
              </div>
              <div className="nb-grid nb-grid--cols-2">
                <div className="nb-cell">
                  <span className="font-mono text-[0.6rem] text-steel">[01] LEXICAL</span>
                  <h3 className="font-display text-xl font-bold my-2 text-foreground">
                    BM25 Search
                  </h3>
                  <p className="text-[0.82rem] text-muted leading-relaxed m-0 mb-6">
                    Full-text lexical search at ~1.2ms p50 with 0.998 recall. Zero infrastructure
                    required.
                  </p>
                  <div className="flex gap-8">
                    <div>
                      <div className="font-display text-2xl font-extrabold text-amber">~1.2ms</div>
                      <div className="font-mono text-[0.55rem] text-steel">P50 LATENCY</div>
                    </div>
                    <div>
                      <div className="font-display text-2xl font-extrabold text-foreground">
                        0.998
                      </div>
                      <div className="font-mono text-[0.55rem] text-steel">RECALL@10</div>
                    </div>
                  </div>
                </div>
                <div className="nb-cell">
                  <span className="font-mono text-[0.6rem] text-steel">[02] VECTOR</span>
                  <h3 className="font-display text-xl font-bold my-2 text-foreground">
                    HNSW Recall
                  </h3>
                  <p className="text-[0.82rem] text-muted leading-relaxed m-0 mb-6">
                    Hierarchical Navigable Small World graphs for approximate nearest neighbor
                    search across vectors.
                  </p>
                  <div className="flex gap-8">
                    <div>
                      <div className="font-display text-2xl font-extrabold text-amber">M=16</div>
                      <div className="font-mono text-[0.55rem] text-steel">CONNECTIONS</div>
                    </div>
                    <div>
                      <div className="font-display text-2xl font-extrabold text-foreground">
                        Cosine
                      </div>
                      <div className="font-mono text-[0.55rem] text-steel">METRIC</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <div className="nb-divider" />

        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-grid nb-grid--cols-2">
              <div className="nb-cell flex flex-col gap-6">
                <div className="nb-frame" data-frame-label="LIVE TOPOLOGY">
                  <div className="flex justify-between items-baseline mb-4">
                    <span className="font-mono text-[0.65rem] text-steel uppercase tracking-[0.05em]">
                      Live Topology
                    </span>
                    <span className="font-mono text-[0.55rem] text-muted">HOVER TO TRAVERSE</span>
                  </div>
                  <GraphTopology />
                </div>
                <div>
                  <h3 className="font-display text-xl font-bold m-0 mb-2">
                    Knowledge Graph Relations
                  </h3>
                  <p className="text-[0.85rem] text-muted leading-relaxed m-0">
                    Hover nodes to explore in-memory relations. VantaDB stores directed adjacency
                    lists alongside vectors for graph-based agent memory applications.
                  </p>
                </div>
              </div>
              <div className="nb-cell flex flex-col gap-6">
                <div className="nb-frame" data-frame-label="RRF WEIGHTS">
                  <RRFWeightsSlider />
                </div>
                <div>
                  <h3 className="font-display text-xl font-bold m-0 mb-2">Dynamic Rank Fusion</h3>
                  <p className="text-[0.85rem] text-muted leading-relaxed m-0">
                    Adjust weights to tune BM25 and HNSW fused recall. VantaDB coordinates sparse
                    token matching and dense vectors at the query level.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </section>

        <div className="nb-divider" />

        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-asymmetric">
              <div>
                <div className="nb-label">02 / 03 — Durability</div>
                <h2 className="font-display text-[clamp(2rem,4vw,3rem)] font-extrabold tracking-[-0.04em] my-5 leading-tight">
                  Crash-Safe WAL
                </h2>
                <p className="font-sans text-[0.95rem] text-muted leading-relaxed">
                  VantaDB guarantees complete transaction safety. Write-Ahead Logging forces log
                  flushes before write acknowledgment, recovering state instantly on reboot.
                </p>
              </div>
              <div className="nb-grid nb-grid--cols-2">
                <div className="nb-cell">
                  <span className="font-mono text-[0.6rem] text-steel">[01] PERSISTENCE</span>
                  <h3 className="font-display text-xl font-bold my-2 text-foreground">
                    WAL Journal
                  </h3>
                  <p className="text-[0.82rem] text-muted leading-relaxed m-0 mb-6">
                    Append-only write journal with CRC32C checks. Zero data corruption on hardware
                    failures.
                  </p>
                  <div className="flex gap-8">
                    <div>
                      <div className="font-display text-2xl font-extrabold text-amber">CRC32C</div>
                      <div className="font-mono text-[0.55rem] text-steel">INTEGRITY</div>
                    </div>
                    <div>
                      <div className="font-display text-2xl font-extrabold text-foreground">
                        Fsync
                      </div>
                      <div className="font-mono text-[0.55rem] text-steel">ON WRITE</div>
                    </div>
                  </div>
                </div>
                <div className="nb-cell">
                  <span className="font-mono text-[0.6rem] text-steel">[02] RESILIENCE</span>
                  <h3 className="font-display text-xl font-bold my-2 text-foreground">
                    Crash Recovery
                  </h3>
                  <p className="text-[0.82rem] text-muted leading-relaxed m-0 mb-6">
                    Automatic log replay during engine bootstrap. Corrupted segments are filtered
                    before reaching memory.
                  </p>
                  <div className="flex gap-8">
                    <div>
                      <div className="font-display text-2xl font-extrabold text-amber">
                        &lt; 1ms
                      </div>
                      <div className="font-mono text-[0.55rem] text-steel">REBOOT RECOVER</div>
                    </div>
                    <div>
                      <div className="font-display text-2xl font-extrabold text-foreground">
                        Auto
                      </div>
                      <div className="font-mono text-[0.55rem] text-steel">REPLAY SCAN</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <div className="nb-divider" />

        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-asymmetric--right">
              <div className="flex flex-col gap-6">
                <div className="nb-frame" data-frame-label="WAL SIMULATOR">
                  <WALSimulator />
                </div>
              </div>
              <div>
                <div className="nb-label">Simulate Integrity</div>
                <h3 className="font-display text-2xl font-extrabold mt-4 mb-2 leading-tight">
                  Test WAL Resilience
                </h3>
                <p className="text-sm text-muted leading-relaxed m-0">
                  Crash the simulator to write unflushed records, then trigger recovery to scan
                  integrity checksums and sync state under 1ms.
                </p>
              </div>
            </div>
          </div>
        </section>

        <ArchitecturePipeline />

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
                    Four modalities. One dependency.
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

      <style>{`
        @media (max-width: 768px) {
          [style*="grid-template-columns: 1fr 1fr"] { grid-template-columns: 1fr !important; }
        }
      `}</style>
    </div>
  );
}

import { createLazyRoute } from "@tanstack/react-router";
import { useEffect, useState, useRef } from "react";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createLazyRoute("/engine")({
  component: EnginePage,
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
      {edges.map(([a, b], i) => (
        <line
          key={i}
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
          key={i}
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
    <div
      className="slider-container"
      style={{ background: "var(--background)", border: "none", padding: 0 }}
    >
      <div
        className="rrf-header"
        style={{
          borderBottom: "1px solid var(--border)",
          paddingBottom: "1rem",
          marginBottom: "1.5rem",
        }}
      >
        <h4
          className="rrf-title"
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "0.8rem",
            textTransform: "uppercase",
            letterSpacing: "0.08em",
          }}
        >
          RRF Weights Planner
        </h4>
        <span
          className="rrf-latency"
          style={{ fontFamily: "var(--font-mono)", fontSize: "0.7rem" }}
        >
          LATENCY: {queryLatency}ms
        </span>
      </div>

      <p
        style={{
          fontSize: "0.85rem",
          color: "var(--muted)",
          lineHeight: 1.5,
          margin: "0 0 1.5rem",
        }}
      >
        Adjust the slider to coordinate keyword recall against vector space clustering.
      </p>

      <div
        className="rrf-labels"
        style={{
          display: "flex",
          justifyContent: "space-between",
          fontFamily: "var(--font-mono)",
          fontSize: "0.7rem",
          fontWeight: 600,
        }}
      >
        <span>BM25: {bm25Weight}%</span>
        <span>HNSW: {hnswWeight}%</span>
      </div>

      <input
        type="range"
        min="0"
        max="100"
        value={bm25Weight}
        onChange={(e) => setBm25Weight(Number(e.target.value))}
        className="rrf-slider"
        style={{ width: "100%", margin: "1.5rem 0" }}
        aria-label="BM25 to HNSW fusion weight ratio"
      />

      <div
        className="rrf-metrics-grid"
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(3, 1fr)",
          gap: "1px",
          background: "var(--border)",
          border: "1px solid var(--border)",
          borderRadius: 0,
          overflow: "hidden",
        }}
      >
        <div className="rrf-metric-card" style={{ background: "var(--surface)", padding: "1rem" }}>
          <div
            className="rrf-metric-label"
            style={{ fontFamily: "var(--font-mono)", fontSize: "0.55rem", color: "var(--steel)" }}
          >
            LEXICAL RECALL
          </div>
          <div
            className="rrf-metric-value"
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "1.1rem",
              fontWeight: 700,
              color: "var(--foreground)",
            }}
          >
            {lexicalRecall}%
          </div>
        </div>
        <div className="rrf-metric-card" style={{ background: "var(--surface)", padding: "1rem" }}>
          <div
            className="rrf-metric-label"
            style={{ fontFamily: "var(--font-mono)", fontSize: "0.55rem", color: "var(--steel)" }}
          >
            VECTOR RECALL
          </div>
          <div
            className="rrf-metric-value"
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "1.1rem",
              fontWeight: 700,
              color: "var(--foreground)",
            }}
          >
            {vectorRecall}%
          </div>
        </div>
        <div
          className="rrf-metric-card"
          style={{
            background: "var(--surface)",
            padding: "1rem",
            borderLeft: "1px solid var(--amber)",
          }}
        >
          <div
            className="rrf-metric-label"
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "0.55rem",
              color: "var(--amber)",
              fontWeight: 700,
            }}
          >
            FUSED @10
          </div>
          <div
            className="rrf-metric-value"
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "1.1rem",
              fontWeight: 700,
              color: "var(--amber)",
            }}
          >
            {fusedRecall}%
          </div>
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
    if (log.includes("!!!")) return "var(--crimson, #ff3b30)";
    if (log.includes("READY") || log.includes("OK")) return "var(--amber)";
    if (log.includes("RECOVER") || log.includes("Syncing")) return "var(--steel)";
    return "var(--muted)";
  };

  return (
    <div
      className="wal-simulator"
      style={{ background: "var(--background)", border: "none", padding: 0 }}
    >
      <div
        className="wal-header"
        style={{
          borderBottom: "1px solid var(--border)",
          paddingBottom: "1rem",
          marginBottom: "1.5rem",
        }}
      >
        <div className="wal-status-group">
          <span
            style={{
              width: 8,
              height: 8,
              background:
                engineState === "ready"
                  ? "var(--amber)"
                  : engineState === "crashed"
                    ? "var(--crimson, #ff3b30)"
                    : "var(--steel)",
              display: "inline-block",
            }}
          />
          <span
            className="wal-status-label"
            style={{ fontFamily: "var(--font-mono)", fontSize: "0.72rem" }}
          >
            STATUS: {engineState}
          </span>
        </div>
        <div className="wal-actions">
          <button
            className="btn-ghost wal-btn-sm"
            onClick={triggerCrash}
            disabled={engineState !== "ready"}
            style={{
              padding: "0.4rem 0.8rem",
              fontSize: "0.65rem",
              fontFamily: "var(--font-mono)",
              textTransform: "uppercase",
            }}
          >
            CRASH ENGINE
          </button>
          <button
            className="btn-primary wal-btn-sm"
            onClick={recoverFromWAL}
            disabled={engineState !== "crashed"}
            style={{
              padding: "0.4rem 0.8rem",
              fontSize: "0.65rem",
              fontFamily: "var(--font-mono)",
              textTransform: "uppercase",
            }}
          >
            RECOVER FROM WAL
          </button>
        </div>
      </div>

      <div
        className="wal-console"
        style={{
          background: "var(--surface)",
          border: "1px solid var(--border)",
          padding: "1.25rem",
          fontFamily: "var(--font-mono)",
          fontSize: "0.7rem",
          height: 180,
          overflowY: "auto",
          lineHeight: 1.6,
        }}
      >
        {logs.map((log, idx) => (
          <div
            key={idx}
            style={{
              color: getLogColor(log),
              borderBottom: "1px solid var(--subtle)",
              padding: "4px 0",
            }}
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
    { name: "GraphRAG", desc: "Traversal", color: "var(--steel)" },
    { name: "WAL", desc: "Durable Write", color: "var(--steel)" },
  ];

  return (
    <section className="engine-section" style={{ borderTop: "1px solid var(--border)" }}>
      <div className="swiss-inner">
        <span className="swiss-eyebrow">03 / 03 — Pipeline</span>
        <h2
          style={{
            fontFamily: "var(--font-display)",
            fontSize: "clamp(2rem, 4vw, 3rem)",
            fontWeight: 800,
            letterSpacing: "-0.04em",
            margin: "1.25rem 0 3rem",
          }}
        >
          End-to-End Query Execution
        </h2>

        <div
          style={{
            overflowX: "auto",
            border: "1px solid var(--border)",
            background: "var(--surface)",
            padding: "3rem 2rem",
          }}
        >
          <div
            style={{ display: "flex", gap: "1rem", alignItems: "center", minWidth: "max-content" }}
          >
            {stages.map((s, i) => (
              <div key={s.name} style={{ display: "flex", alignItems: "center", gap: "1rem" }}>
                <div
                  style={{
                    border: `1px solid ${s.color}`,
                    padding: "1.25rem 2rem",
                    background: "var(--background)",
                    minWidth: 140,
                    textAlign: "left",
                  }}
                >
                  <div
                    style={{
                      fontFamily: "var(--font-display)",
                      fontSize: "0.9rem",
                      fontWeight: 700,
                      color: "var(--foreground)",
                    }}
                  >
                    {s.name}
                  </div>
                  <div
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.55rem",
                      color: "var(--muted)",
                      textTransform: "uppercase",
                      marginTop: "0.25rem",
                    }}
                  >
                    {s.desc}
                  </div>
                </div>
                {i < stages.length - 1 && (
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      color: "var(--border)",
                      fontSize: "1rem",
                    }}
                  >
                    →
                  </span>
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
    <div className="engine-page">
      <SwissSubpageHero
        num="01"
        eyebrow="Core Engine"
        title={
          <span>
            Four modalities.
            <br />
            One atomic contract.
          </span>
        }
        sub="VantaDB consolidates lexical matching, HNSW vector search, Graph traversal nodes and transactional persistence in a zero-dependency Rust local-first library database."
      />

      <main className="engine-main">
        <section className="engine-section engine-section--bordered">
          <div className="swiss-grid-12" style={{ alignItems: "start" }}>
            <div className="col-span-4">
              <span className="swiss-eyebrow">01 / 03 — Hybrid Search</span>
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "clamp(2rem, 4vw, 3rem)",
                  fontWeight: 800,
                  letterSpacing: "-0.04em",
                  margin: "1.25rem 0",
                  lineHeight: 1.05,
                }}
              >
                BM25 + HNSW + RRF
              </h2>
              <p
                style={{
                  fontFamily: "var(--font-sans)",
                  fontSize: "0.95rem",
                  color: "var(--muted)",
                  lineHeight: 1.6,
                }}
              >
                VantaDB query planner optimizes combined metadata filters, HNSW vector similarity,
                and BM25 full-text queries, synthesizing them into a single-pass execution plan.
              </p>
            </div>

            <div
              className="col-span-8"
              style={{
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                gap: "1px",
                background: "var(--border)",
                border: "1px solid var(--border)",
              }}
            >
              <div style={{ background: "var(--background)", padding: "2.5rem 2rem" }}>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--steel)",
                  }}
                >
                  [01] LEXICAL
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.25rem",
                    fontWeight: 700,
                    margin: "0.5rem 0",
                    color: "var(--foreground)",
                  }}
                >
                  BM25 Search
                </h3>
                <p
                  style={{
                    fontSize: "0.82rem",
                    color: "var(--muted)",
                    lineHeight: 1.5,
                    margin: "0 0 1.5rem",
                  }}
                >
                  Full-text lexical search at ~1.2ms p50 with 0.998 recall. Zero infrastructure
                  required.
                </p>
                <div style={{ display: "flex", gap: "2rem" }}>
                  <div>
                    <div
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "1.5rem",
                        fontWeight: 800,
                        color: "var(--amber)",
                      }}
                    >
                      ~1.2ms
                    </div>
                    <div
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.55rem",
                        color: "var(--steel)",
                      }}
                    >
                      P50 LATENCY
                    </div>
                  </div>
                  <div>
                    <div
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "1.5rem",
                        fontWeight: 800,
                        color: "var(--foreground)",
                      }}
                    >
                      0.998
                    </div>
                    <div
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.55rem",
                        color: "var(--steel)",
                      }}
                    >
                      RECALL@10
                    </div>
                  </div>
                </div>
              </div>

              <div style={{ background: "var(--background)", padding: "2.5rem 2rem" }}>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--steel)",
                  }}
                >
                  [02] VECTOR
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.25rem",
                    fontWeight: 700,
                    margin: "0.5rem 0",
                    color: "var(--foreground)",
                  }}
                >
                  HNSW Recall
                </h3>
                <p
                  style={{
                    fontSize: "0.82rem",
                    color: "var(--muted)",
                    lineHeight: 1.5,
                    margin: "0 0 1.5rem",
                  }}
                >
                  Hierarchical Navigable Small World graphs for approximate nearest neighbor search
                  across vectors.
                </p>
                <div style={{ display: "flex", gap: "2rem" }}>
                  <div>
                    <div
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "1.5rem",
                        fontWeight: 800,
                        color: "var(--amber)",
                      }}
                    >
                      M=16
                    </div>
                    <div
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.55rem",
                        color: "var(--steel)",
                      }}
                    >
                      CONNECTIONS
                    </div>
                  </div>
                  <div>
                    <div
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "1.5rem",
                        fontWeight: 800,
                        color: "var(--foreground)",
                      }}
                    >
                      Cosine
                    </div>
                    <div
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.55rem",
                        color: "var(--steel)",
                      }}
                    >
                      METRIC
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="engine-section engine-section--bordered">
          <div className="swiss-grid-12" style={{ alignItems: "start" }}>
            <div
              className="col-span-6"
              style={{ display: "flex", flexDirection: "column", gap: "1.5rem" }}
            >
              <div
                style={{
                  border: "1px solid var(--border)",
                  padding: "2rem",
                  background: "var(--surface)",
                }}
              >
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "baseline",
                    marginBottom: "1rem",
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
                    Live Topology
                  </span>
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.55rem",
                      color: "var(--muted)",
                    }}
                  >
                    HOVER TO TRAVERSE
                  </span>
                </div>
                <GraphTopology />
              </div>
              <div>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.25rem",
                    fontWeight: 700,
                    margin: "0 0 0.5rem",
                  }}
                >
                  Knowledge Graph Relations
                </h3>
                <p
                  style={{ fontSize: "0.85rem", color: "var(--muted)", lineHeight: 1.6, margin: 0 }}
                >
                  Hover nodes to explore in-memory relations. VantaDB stores directed adjacency
                  lists alongside vectors for seamless GraphRAG implementations.
                </p>
              </div>
            </div>

            <div
              className="col-span-6"
              style={{ display: "flex", flexDirection: "column", gap: "1.5rem" }}
            >
              <div
                style={{
                  border: "1px solid var(--border)",
                  padding: "2rem",
                  background: "var(--surface)",
                }}
              >
                <RRFWeightsSlider />
              </div>
              <div>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.25rem",
                    fontWeight: 700,
                    margin: "0 0 0.5rem",
                  }}
                >
                  Dynamic Rank Fusion
                </h3>
                <p
                  style={{ fontSize: "0.85rem", color: "var(--muted)", lineHeight: 1.6, margin: 0 }}
                >
                  Adjust weights to tune BM25 and HNSW fused recall. VantaDB coordinates sparse
                  token matching and dense vectors at the query level.
                </p>
              </div>
            </div>
          </div>
        </section>

        <section className="engine-section engine-section--bordered">
          <div className="swiss-grid-12" style={{ alignItems: "start" }}>
            <div className="col-span-4">
              <span className="swiss-eyebrow">02 / 03 — Durability</span>
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "clamp(2rem, 4vw, 3rem)",
                  fontWeight: 800,
                  letterSpacing: "-0.04em",
                  margin: "1.25rem 0",
                  lineHeight: 1.05,
                }}
              >
                Crash-Safe WAL
              </h2>
              <p
                style={{
                  fontFamily: "var(--font-sans)",
                  fontSize: "0.95rem",
                  color: "var(--muted)",
                  lineHeight: 1.6,
                }}
              >
                VantaDB guarantees complete transaction safety. Write-Ahead Logging forces log
                flushes before write acknowledgment, recovering state instantly on reboot.
              </p>
            </div>

            <div
              className="col-span-8"
              style={{
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                gap: "1px",
                background: "var(--border)",
                border: "1px solid var(--border)",
              }}
            >
              <div style={{ background: "var(--background)", padding: "2.5rem 2rem" }}>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--steel)",
                  }}
                >
                  [01] PERSISTENCE
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.25rem",
                    fontWeight: 700,
                    margin: "0.5rem 0",
                    color: "var(--foreground)",
                  }}
                >
                  WAL Journal
                </h3>
                <p
                  style={{
                    fontSize: "0.82rem",
                    color: "var(--muted)",
                    lineHeight: 1.5,
                    margin: "0 0 1.5rem",
                  }}
                >
                  Append-only write journal with CRC32C checks. Zero data corruption on hardware
                  failures.
                </p>
                <div style={{ display: "flex", gap: "2rem" }}>
                  <div>
                    <div
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "1.5rem",
                        fontWeight: 800,
                        color: "var(--amber)",
                      }}
                    >
                      CRC32C
                    </div>
                    <div
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.55rem",
                        color: "var(--steel)",
                      }}
                    >
                      INTEGRITY
                    </div>
                  </div>
                  <div>
                    <div
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "1.5rem",
                        fontWeight: 800,
                        color: "var(--foreground)",
                      }}
                    >
                      Fsync
                    </div>
                    <div
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.55rem",
                        color: "var(--steel)",
                      }}
                    >
                      ON WRITE
                    </div>
                  </div>
                </div>
              </div>

              <div style={{ background: "var(--background)", padding: "2.5rem 2rem" }}>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--steel)",
                  }}
                >
                  [02] RESILIENCE
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.25rem",
                    fontWeight: 700,
                    margin: "0.5rem 0",
                    color: "var(--foreground)",
                  }}
                >
                  Crash Recovery
                </h3>
                <p
                  style={{
                    fontSize: "0.82rem",
                    color: "var(--muted)",
                    lineHeight: 1.5,
                    margin: "0 0 1.5rem",
                  }}
                >
                  Automatic log replay during engine bootstrap. Corrupted segments are filtered
                  before reaching memory.
                </p>
                <div style={{ display: "flex", gap: "2rem" }}>
                  <div>
                    <div
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "1.5rem",
                        fontWeight: 800,
                        color: "var(--amber)",
                      }}
                    >
                      &lt; 1ms
                    </div>
                    <div
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.55rem",
                        color: "var(--steel)",
                      }}
                    >
                      REBOOT RECOVER
                    </div>
                  </div>
                  <div>
                    <div
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "1.5rem",
                        fontWeight: 800,
                        color: "var(--foreground)",
                      }}
                    >
                      Auto
                    </div>
                    <div
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.55rem",
                        color: "var(--steel)",
                      }}
                    >
                      REPLAY SCAN
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="engine-section engine-section--bordered">
          <div className="swiss-grid-12" style={{ alignItems: "start" }}>
            <div className="col-span-8">
              <div
                style={{
                  border: "1px solid var(--border)",
                  padding: "2rem",
                  background: "var(--surface)",
                }}
              >
                <WALSimulator />
              </div>
            </div>
            <div className="col-span-4">
              <span className="swiss-eyebrow">Simulate Integrity</span>
              <h3
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "1.5rem",
                  fontWeight: 800,
                  margin: "1rem 0 0.5rem",
                  lineHeight: 1.1,
                }}
              >
                Test WAL Resilience
              </h3>
              <p
                style={{ fontSize: "0.875rem", color: "var(--muted)", lineHeight: 1.6, margin: 0 }}
              >
                Crash the simulator to write unflushed records, then trigger recovery to scan
                integrity checksums and sync state under 1ms.
              </p>
            </div>
          </div>
        </section>

        <ArchitecturePipeline />
      </main>
    </div>
  );
}

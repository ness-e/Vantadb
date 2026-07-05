import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/use-cases")({
  component: UseCasesPage,
  pendingComponent: PendingComponent,
});

const CASES = [
  {
    num: "01",
    title: "AI Agent Memory",
    tags: ["persistent", "≤1ms read", "crash-safe"],
    desc: "Store conversational history, agent thoughts, and user preferences locally. Context survives restarts with WAL crash safety — no external database needed.",
  },
  {
    num: "02",
    title: "Local-First RAG",
    tags: ["hybrid search", "zero deps", "on-device"],
    desc: "Run BM25 lexical + HNSW vector fusion in-process. No external server, no network overhead — full hybrid search without spinning up a container.",
  },
  {
    num: "03",
    title: "Codebase Intelligence",
    tags: ["graph edges", "AST-aware", "30K loc/s"],
    desc: "Map function definitions, imports, and caller relations in a local knowledge graph. Traverse graph hops with vector similarity for accurate code retrieval.",
  },
  {
    num: "04",
    title: "Multi-Agent Orchestration",
    tags: ["namespaces", "isolation", "concurrent"],
    desc: "Run hundreds of independent agents on a single DB file. Namespace-level isolation prevents key collisions while keeping a unified storage footprint.",
  },
  {
    num: "05",
    title: "E-Commerce Semantic Search",
    tags: ["vector", "metadata filter", "real-time"],
    desc: "Serve personalized product recommendations using vector similarity on behavior embeddings — updated in real-time with zero reindexing.",
  },
  {
    num: "06",
    title: "Edge / IoT Inference",
    tags: ["embedded", "ARM/RISC-V", "WAL-safe"],
    desc: "Persist device state and sensor telemetry on embedded hardware. Sub-millisecond reads with CRC32C WAL crash protection on ARM and RISC-V.",
  },
  {
    num: "07",
    title: "Healthcare RAG",
    tags: ["PHI-safe", "audit log", "zero-server"],
    desc: "Run private medical RAG on-device with full audit trails. Embeddings and FHIR documents stay local — no PHI leaves the hospital network.",
  },
  {
    num: "08",
    title: "Financial Document Processing",
    tags: ["compliance", "high-throughput", "WAL"],
    desc: "Parse, index, and search invoices, statements, and regulatory filings with crash-safe durability. Thousands of documents per second on a single thread.",
  },
];

const PIPELINE_STEPS = [
  {
    num: "01",
    title: "Memory",
    desc: "Embeddings and metadata are written to the LSM-tree engine with immediate durability. The WAL guarantees crash recovery — every vector, every key, every byte.",
    tags: ["write-ahead log", "CRC32C", "O(1) append"],
  },
  {
    num: "02",
    title: "Search",
    desc: "HNSW vector index + BM25 lexical index fused via RRF in a single call. Sub-millisecond latency with zero network — all computation is in-process.",
    tags: ["HNSW", "BM25", "RRF fusion"],
  },
  {
    num: "03",
    title: "Persist",
    desc: "Everything lives in a single portable DB file. Backup via SCP, move across machines, survive process kills — no reindexing, no restore procedure.",
    tags: ["single file", "portable", "zero reindex"],
  },
];

function UseCasesPage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="04"
        eyebrow="Use Cases"
        title={
          <span>
            Built for agents
            <br />
            that need context.
          </span>
        }
        sub="Eight production-tested patterns for persistent memory, hybrid search, and agentic data — all running in-process with zero external dependencies."
      />

      <main className="nb-main">
        <section className="nb-section nb-section--bordered">
          <span className="nb-eyebrow">01 / 02 — Production Patterns</span>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(2, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {CASES.map((c) => (
              <div
                key={c.num}
                className="usecase-card"
                style={{
                  padding: "2.5rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.75rem",
                  transition: "background-color 150ms var(--ease-cut)",
                  background: "var(--background)",
                }}
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
                      color: "var(--amber)",
                      textTransform: "uppercase",
                      letterSpacing: "0.08em",
                    }}
                  >
                    {c.num}
                  </span>
                </div>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1rem",
                    fontWeight: 700,
                    letterSpacing: "-0.02em",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {c.title}
                </h3>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.8rem",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    margin: 0,
                    flex: 1,
                  }}
                >
                  {c.desc}
                </p>
                <div style={{ display: "flex", gap: "0.4rem", flexWrap: "wrap" }}>
                  {c.tags.map((tag) => (
                    <span
                      key={tag}
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.55rem",
                        fontWeight: 700,
                        textTransform: "uppercase",
                        letterSpacing: "0.08em",
                        color: "var(--steel)",
                        border: "1px solid var(--border)",
                        padding: "0.2rem 0.5rem",
                      }}
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </section>

        <section className="nb-section">
          <span className="nb-eyebrow">02 / 02 — Core Pipeline</span>
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "clamp(1.5rem, 3vw, 2rem)",
              fontWeight: 800,
              letterSpacing: "-0.04em",
              margin: "1.25rem 0 3rem",
              lineHeight: 1.05,
            }}
          >
            Memory → Search → Persist.
          </h2>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
            }}
          >
            {PIPELINE_STEPS.map((step) => (
              <div
                key={step.num}
                style={{
                  background: "var(--background)",
                  padding: "2.5rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.75rem",
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "2rem",
                    fontWeight: 800,
                    color: "var(--border)",
                    lineHeight: 1,
                    letterSpacing: "-0.04em",
                  }}
                >
                  {step.num}
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.2rem",
                    fontWeight: 700,
                    letterSpacing: "-0.03em",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {step.title}
                </h3>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.8rem",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    margin: 0,
                    flex: 1,
                  }}
                >
                  {step.desc}
                </p>
                <div style={{ display: "flex", gap: "0.4rem", flexWrap: "wrap" }}>
                  {step.tags.map((tag) => (
                    <span
                      key={tag}
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.55rem",
                        fontWeight: 700,
                        textTransform: "uppercase",
                        letterSpacing: "0.08em",
                        color: "var(--amber)",
                        border: "1px solid rgba(255, 85, 0, 0.3)",
                        padding: "0.2rem 0.5rem",
                      }}
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </section>
      </main>

      <style>{`
        .usecase-card:hover {
          background: var(--surface-raised) !important;
        }
        @media (max-width: 768px) {
          [style*="grid-template-columns: repeat(2, 1fr)"] { grid-template-columns: 1fr !important; }
          [style*="grid-template-columns: repeat(3, 1fr)"] { grid-template-columns: 1fr !important; }
        }
      `}</style>
    </div>
  );
}

import { createLazyRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/storage")({
  component: StoragePage,
  pendingComponent: PendingComponent,
});

// ── Data ─────────────────────────────────────────────────────────────────────
const LEGACY_STACK = [
  "Pinecone: vector index (proprietary, cloud-only)",
  "Redis: metadata + cache (in-memory, not durable)",
  "S3: document storage + backups",
  "Three different SDKs, auth mechanisms, latencies",
  "No transactional consistency across services",
];

const VANTA_STACK = [
  "HNSW vector index: native, configurable recall",
  "BM25 full-text: tokenized, stemmed, scored",
  "WAL-backed durability: crash-safe, no data loss",
  "Single SDK: `pip install vantadb-py`, one `connect()`",
];

const LAYERS = [
  {
    num: "01",
    title: "LSM-Tree Foundation",
    desc: "Columnar storage with log-structured merge-tree compaction. Typed, nullable columns with efficient compression and predicate pushdown.",
  },
  {
    num: "02",
    title: "HNSW Vector Index",
    desc: "Hierarchical navigable small world graphs for approximate nearest neighbor search. Configurable M (16–64) and efConstruction for recall/speed tradeoffs.",
  },
  {
    num: "03",
    title: "WAL Durability",
    desc: "Write-ahead log with checksum verification and automatic recovery. Configurable sync modes: async (fastest), fsync (safe), or full (maximum durability).",
  },
];

function StoragePage() {
  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="09"
        eyebrow="Storage Architecture"
        title={
          <span>
            Single binary.
            <br />
            Three engines.
          </span>
        }
        sub="No more juggling Pinecone, Redis, and S3. VantaDB unifies vector, full-text, and structured storage in one embedded engine with WAL-backed durability."
      />

      <main className="engine-main">
        {/* Section 1: Stack Comparison */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 03 — Stack Consolidation</span>

          {/* Visual: 3 → 1 */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 60px 1fr",
              gap: "0",
              alignItems: "center",
              marginTop: "3rem",
              marginBottom: "3rem",
            }}
          >
            {/* Legacy */}
            <div
              style={{
                border: "1px solid var(--border)",
                display: "flex",
                flexDirection: "column",
                gap: "1px",
                background: "var(--border)",
              }}
            >
              <div style={{ background: "var(--surface)", padding: "0.75rem 1.5rem" }}>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--steel)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                  }}
                >
                  Legacy — 3 services
                </span>
              </div>
              {[
                { name: "Pinecone", role: "Vector index", color: "#ff3b30" },
                { name: "Redis", role: "Cache + metadata", color: "#ff9500" },
                { name: "S3", role: "Document storage", color: "var(--steel)" },
              ].map((s) => (
                <div
                  key={s.name}
                  style={{
                    background: "var(--background)",
                    padding: "1.25rem 1.5rem",
                    borderLeft: `3px solid ${s.color}`,
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
                      fontSize: "0.6rem",
                      color: "var(--steel)",
                      textTransform: "uppercase",
                      letterSpacing: "0.06em",
                      marginTop: "0.2rem",
                    }}
                  >
                    {s.role}
                  </div>
                </div>
              ))}
              <div style={{ background: "var(--surface)", padding: "0.75rem 1.5rem" }}>
                <span
                  style={{ fontFamily: "var(--font-mono)", fontSize: "0.6rem", color: "#ff3b30" }}
                >
                  3 SDKs · 3 bills · $200/mo
                </span>
              </div>
            </div>

            {/* Arrow */}
            <div style={{ display: "flex", justifyContent: "center", alignItems: "center" }}>
              <svg
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="var(--amber)"
                strokeWidth="1.5"
              >
                <path d="M5 12h14m-4-4 4 4-4 4" />
              </svg>
            </div>

            {/* VantaDB */}
            <div
              style={{
                border: "1px solid var(--border)",
                borderLeft: "2px solid var(--amber)",
                display: "flex",
                flexDirection: "column",
                gap: "1px",
                background: "var(--border)",
              }}
            >
              <div style={{ background: "var(--surface-raised)", padding: "0.75rem 1.5rem" }}>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--amber)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                  }}
                >
                  VantaDB — 1 binary
                </span>
              </div>
              <div
                style={{
                  background: "var(--surface)",
                  padding: "2.5rem 1.5rem",
                  textAlign: "left",
                }}
              >
                <div
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.4rem",
                    fontWeight: 800,
                    letterSpacing: "-0.04em",
                    color: "var(--amber)",
                  }}
                >
                  vantadb
                </div>
                <div
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.65rem",
                    color: "var(--muted)",
                    marginTop: "0.5rem",
                  }}
                >
                  Vector · Full-text · Hybrid · WAL
                </div>
              </div>
              <div style={{ background: "var(--surface-raised)", padding: "0.75rem 1.5rem" }}>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--amber)",
                  }}
                >
                  1 SDK · 1 install · $0
                </span>
              </div>
            </div>
          </div>

          {/* Side-by-side bullets */}
          <div className="swiss-grid-12" style={{ alignItems: "start", gap: "1px" }}>
            <div
              className="col-span-6"
              style={{ border: "1px solid var(--border)", padding: "2.5rem" }}
            >
              <h3
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "0.9rem",
                  fontWeight: 800,
                  textTransform: "uppercase",
                  letterSpacing: "0.05em",
                  color: "var(--steel)",
                  marginBottom: "1.25rem",
                }}
              >
                Legacy stack
              </h3>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.75rem",
                }}
              >
                {LEGACY_STACK.map((item, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.8rem",
                      color: "var(--muted)",
                      lineHeight: 1.5,
                    }}
                  >
                    <span
                      style={{
                        color: "#ff3b30",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
                      }}
                    >
                      ✗
                    </span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div
              className="col-span-6"
              style={{
                border: "1px solid var(--border)",
                borderLeft: "2px solid var(--amber)",
                padding: "2.5rem",
                background: "var(--surface)",
              }}
            >
              <h3
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "0.9rem",
                  fontWeight: 800,
                  textTransform: "uppercase",
                  letterSpacing: "0.05em",
                  color: "var(--amber)",
                  marginBottom: "1.25rem",
                }}
              >
                VantaDB
              </h3>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.75rem",
                }}
              >
                {VANTA_STACK.map((item, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.8rem",
                      color: "var(--foreground)",
                      lineHeight: 1.5,
                    }}
                  >
                    <span
                      style={{
                        color: "var(--amber)",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
                      }}
                    >
                      ✓
                    </span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </section>

        {/* Section 2: Architecture layers */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">02 / 03 — Storage Architecture</span>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {LAYERS.map((l) => (
              <div
                key={l.num}
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
                    fontSize: "0.6rem",
                    color: "var(--amber)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                  }}
                >
                  {l.num}
                </span>
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
                  {l.title}
                </h3>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.8rem",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    margin: 0,
                  }}
                >
                  {l.desc}
                </p>
              </div>
            ))}
          </div>
        </section>

        {/* Section 3: Engine states */}
        <section className="engine-section">
          <span className="swiss-eyebrow">03 / 03 — Engine States</span>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {[
              {
                state: "Active",
                desc: "Serving reads + writes. HNSW index is warm, WAL is accepting commits, BM25 index is queryable.",
              },
              {
                state: "Checkpoint",
                desc: "WAL flush + index compaction. Triggered every N writes or after timeout. Transparent to reads.",
              },
              {
                state: "Recovery",
                desc: "WAL replay on restart. Automatic — no manual intervention. Crash-safe by design.",
              },
            ].map((s) => (
              <div key={s.state} style={{ background: "var(--background)", padding: "2.5rem" }}>
                <div
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--amber)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                    marginBottom: "0.5rem",
                  }}
                >
                  STATE
                </div>
                <div
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.4rem",
                    fontWeight: 800,
                    letterSpacing: "-0.04em",
                    color: "var(--foreground)",
                    marginBottom: "0.75rem",
                  }}
                >
                  {s.state}
                </div>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.8rem",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    margin: 0,
                  }}
                >
                  {s.desc}
                </p>
              </div>
            ))}
          </div>
        </section>
      </main>

      <style>{`
        @media (max-width: 768px) {
          .swiss-grid-12 { grid-template-columns: 1fr !important; }
          [style*="grid-template-columns: 1fr 60px 1fr"] { grid-template-columns: 1fr !important; gap: 1rem !important; }
          [style*="grid-template-columns: repeat(3, 1fr)"] { grid-template-columns: 1fr !important; }
        }
      `}</style>
    </div>
  );
}

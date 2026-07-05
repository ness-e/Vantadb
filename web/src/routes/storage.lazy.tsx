import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/storage")({
  component: StoragePage,
  pendingComponent: PendingComponent,
});

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
    <div>
      <NbSubpageHero
        num="09"
        title={
          <span>
            Single binary.
            <br />
            Three engines.
          </span>
        }
        sub="No more juggling Pinecone, Redis, and S3. VantaDB unifies vector, full-text, and structured storage in one embedded engine with WAL-backed durability."
      />

      <main>
        <section className="nb-section">
          <div className="nb-inner">
            <h2
              style={{
                fontFamily: "var(--font-display)",
                fontSize: "var(--text-title)",
                fontWeight: 700,
                color: "var(--foreground)",
                margin: "0 0 var(--space-md)",
              }}
            >
              Stack Consolidation
            </h2>

            <div className="grid grid-cols-[1fr_60px_1fr] items-center mt-12 mb-12">
              <div className="border-2 border-[var(--border-visible)] flex flex-col">
                <div
                  className="nb-cell"
                  style={{ borderBottom: "1px solid var(--border-visible)" }}
                >
                  <span className="font-mono text-[0.6rem] text-steel uppercase tracking-[0.08em]">
                    Legacy — 3 services
                  </span>
                </div>
                {[
                  { name: "Pinecone", role: "Vector index", color: "var(--danger)" },
                  { name: "Redis", role: "Cache + metadata", color: "#ff9500" },
                  { name: "S3", role: "Document storage", color: "var(--steel)" },
                ].map((s) => (
                  <div
                    key={s.name}
                    className="nb-cell"
                    style={{ borderLeft: `3px solid ${s.color}` }}
                  >
                    <div className="font-display text-[0.9rem] font-bold text-foreground">
                      {s.name}
                    </div>
                    <div className="font-mono text-[0.6rem] text-steel uppercase tracking-[0.06em] mt-1">
                      {s.role}
                    </div>
                  </div>
                ))}
                <div className="nb-cell" style={{ borderTop: "1px solid var(--border-visible)" }}>
                  <span className="font-mono text-[0.6rem]" style={{ color: "var(--danger)" }}>
                    3 SDKs · 3 bills · $200/mo
                  </span>
                </div>
              </div>

              <div className="flex justify-center items-center">
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

              <div className="border-2 border-[var(--border-visible)] border-l-[var(--amber)] flex flex-col">
                <div
                  className="nb-cell"
                  style={{ borderBottom: "1px solid var(--border-visible)" }}
                >
                  <span className="font-mono text-[0.6rem] text-amber uppercase tracking-[0.08em]">
                    VantaDB — 1 binary
                  </span>
                </div>
                <div className="nb-cell">
                  <div className="font-display text-[1.4rem] font-extrabold tracking-[-0.04em] text-amber">
                    vantadb
                  </div>
                  <div className="font-mono text-[0.65rem] text-muted mt-2">
                    Vector · Full-text · Hybrid · WAL
                  </div>
                </div>
                <div className="nb-cell" style={{ borderTop: "1px solid var(--border-visible)" }}>
                  <span className="font-mono text-[0.6rem] text-amber">1 SDK · 1 install · $0</span>
                </div>
              </div>
            </div>

            <div className="nb-grid nb-grid--cols-2">
              <div className="nb-cell">
                <div
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-label)",
                    color: "var(--steel)",
                    fontWeight: 700,
                    marginBottom: "var(--space-sm)",
                  }}
                >
                  LEGACY STACK
                </div>
                <ul
                  className="flex flex-col gap-3 mt-4"
                  style={{ listStyle: "none", margin: 0, padding: 0 }}
                >
                  {LEGACY_STACK.map((item) => (
                    <li key={item} className="flex gap-3 text-sm text-muted leading-relaxed">
                      <span
                        className="font-mono font-bold flex-shrink-0"
                        style={{ color: "var(--danger)" }}
                      >
                        ✗
                      </span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
              <div className="nb-cell" style={{ borderLeft: "2px solid var(--amber)" }}>
                <div
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-label)",
                    color: "var(--amber)",
                    fontWeight: 700,
                    marginBottom: "var(--space-sm)",
                  }}
                >
                  VANTADB
                </div>
                <ul
                  className="flex flex-col gap-3 mt-4"
                  style={{ listStyle: "none", margin: 0, padding: 0 }}
                >
                  {VANTA_STACK.map((item) => (
                    <li key={item} className="flex gap-3 text-sm text-foreground leading-relaxed">
                      <span className="font-mono font-bold flex-shrink-0 text-amber">✓</span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        </section>

        <section className="nb-section">
          <div className="nb-inner">
            <h2
              style={{
                fontFamily: "var(--font-display)",
                fontSize: "var(--text-title)",
                fontWeight: 700,
                color: "var(--foreground)",
                margin: "0 0 var(--space-md)",
              }}
            >
              Storage Architecture
            </h2>

            <div className="nb-grid nb-grid--cols-3" style={{ marginTop: "3rem" }}>
              {LAYERS.map((l) => (
                <div key={l.num} className="nb-cell flex flex-col gap-3">
                  <span className="font-mono text-[0.6rem] text-amber uppercase tracking-[0.08em]">
                    {l.num}
                  </span>
                  <h3 className="font-display text-base font-bold tracking-[-0.02em] text-foreground m-0">
                    {l.title}
                  </h3>
                  <p className="text-sm text-muted leading-relaxed m-0">{l.desc}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="nb-section">
          <div className="nb-inner">
            <h2
              style={{
                fontFamily: "var(--font-display)",
                fontSize: "var(--text-title)",
                fontWeight: 700,
                color: "var(--foreground)",
                margin: "0 0 var(--space-md)",
              }}
            >
              Engine States
            </h2>

            <div className="nb-grid nb-grid--cols-3" style={{ marginTop: "3rem" }}>
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
                <div key={s.state} className="nb-cell">
                  <div className="font-mono text-[0.6rem] text-amber uppercase tracking-[0.08em] mb-2">
                    STATE
                  </div>
                  <div className="font-display text-[1.4rem] font-extrabold tracking-[-0.04em] text-foreground mb-3">
                    {s.state}
                  </div>
                  <p className="text-sm text-muted leading-relaxed m-0">{s.desc}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="nb-section nb-bg-dot">
          <div className="nb-inner">
            <div className="nb-block-amber">
              <div className="flex items-center justify-between gap-4 flex-wrap">
                <div>
                  <h2
                    className="font-display text-2xl font-extrabold"
                    style={{ color: "var(--text-on-amber)" }}
                  >
                    One binary. Three engines. Zero ops.
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
          [style*="grid-template-columns: 1fr 60px 1fr"] { grid-template-columns: 1fr !important; gap: 1rem !important; }
        }
      `}</style>
    </div>
  );
}

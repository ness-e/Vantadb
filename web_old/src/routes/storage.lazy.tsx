import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbBlockAmber, NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import "../styles/storage.css";

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
    title: "LSM-Tree Foundation",
    desc: "Columnar storage with log-structured merge-tree compaction. Typed, nullable columns with efficient compression and predicate pushdown.",
  },
  {
    title: "HNSW Vector Index",
    desc: "Hierarchical navigable small world graphs for approximate nearest neighbor search. Configurable M (16\u201364) and efConstruction for recall/speed tradeoffs.",
  },
  {
    title: "WAL Durability",
    desc: "Write-ahead log with checksum verification and automatic recovery. Configurable sync modes: async (fastest), fsync (safe), or full (maximum durability).",
  },
];

const ENGINE_STATES = [
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
    desc: "WAL replay on restart. Automatic \u2014 no manual intervention. Crash-safe by design.",
  },
];

const LEGACY_SERVICES = [
  { name: "Pinecone", role: "Vector index", color: "var(--danger)" },
  { name: "Redis", role: "Cache + metadata", color: "#ff9500" },
  { name: "S3", role: "Document storage", color: "var(--steel)" },
];

function StoragePage() {
  const stackRef = useRef<HTMLElement>(null);
  const archRef = useRef<HTMLElement>(null);
  const statesRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = stackRef.current?.querySelectorAll<HTMLElement>(".nc-stor-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, stackRef);

  useAnimationSafe(() => {
    const parts = archRef.current?.querySelectorAll<HTMLElement>(".nc-stor-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, archRef);

  useAnimationSafe(() => {
    const parts = statesRef.current?.querySelectorAll<HTMLElement>(".nc-stor-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, statesRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p04"
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
        <NbSection ref={stackRef} ariaLabel="Stack consolidation">
          <NbSectionHeader
            monoLabel="[STACK CONSOLIDATION]"
            headline="Three services to one binary."
            sub="Replace your entire retrieval stack \u2014 Pinecone, Redis, S3 \u2014 with a single embedded engine."
          />

          <div className="nc-stor-compare nc-stor-part">
            <div className="nc-stor-col">
              <div className="nc-stor-cell">
                <span className="nc-stor-label nc-stor-label--steel">LEGACY \u2014 3 services</span>
              </div>
              {LEGACY_SERVICES.map((s) => (
                <div
                  key={s.name}
                  className="nc-stor-cell"
                  style={{ borderLeft: `3px solid ${s.color}` }}
                >
                  <div className="nc-stor-service">{s.name}</div>
                  <div className="nc-stor-role">{s.role}</div>
                </div>
              ))}
              <div className="nc-stor-cell">
                <span className="nc-stor-label nc-stor-label--danger">
                  3 SDKs \u00B7 3 bills \u00B7 $200/mo
                </span>
              </div>
            </div>

            <div className="nc-stor-arrow">
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

            <div className="nc-stor-col nc-stor-col--vanta">
              <div className="nc-stor-cell">
                <span className="nc-stor-label nc-stor-label--amber">VantaDB \u2014 1 binary</span>
              </div>
              <div className="nc-stor-cell">
                <div className="nc-stor-brand">vantadb</div>
                <div className="nc-stor-tags">Vector \u00B7 Full-text \u00B7 Hybrid \u00B7 WAL</div>
              </div>
              <div className="nc-stor-cell">
                <span className="nc-stor-label nc-stor-label--amber">
                  1 SDK \u00B7 1 install \u00B7 $0
                </span>
              </div>
            </div>
          </div>

          <div className="nc-stor-compare nc-stor-part">
            <div className="nc-stor-col">
              <span
                className="nc-stor-label nc-stor-label--steel"
                style={{ display: "block", padding: "var(--space-md)" }}
              >
                LEGACY STACK
              </span>
              <ul className="nc-stor-list" style={{ padding: "0 var(--space-md) var(--space-md)" }}>
                {LEGACY_STACK.map((item) => (
                  <li key={item} className="nc-stor-item nc-stor-item--muted">
                    <span className="nc-stor-icon nc-stor-icon--danger">\u2717</span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nc-stor-arrow" />
            <div className="nc-stor-col nc-stor-col--vanta">
              <span
                className="nc-stor-label nc-stor-label--amber"
                style={{ display: "block", padding: "var(--space-md)" }}
              >
                VANTADB
              </span>
              <ul className="nc-stor-list" style={{ padding: "0 var(--space-md) var(--space-md)" }}>
                {VANTA_STACK.map((item) => (
                  <li key={item} className="nc-stor-item nc-stor-item--fg">
                    <span className="nc-stor-icon nc-stor-icon--amber">\u2713</span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </NbSection>

        <NbSection ref={archRef} ariaLabel="Storage architecture">
          <NbSectionHeader
            monoLabel="[STORAGE ARCHITECTURE]"
            headline="Three engines, one foundation."
            sub="VantaDB\u2019s storage stack is built on an LSM-tree foundation with dedicated vector and full-text indexes."
          />

          <div className="nc-stor-arch nc-stor-part">
            {LAYERS.map((l, i) => (
              <div key={i} className="nc-stor-arch-card">
                <span className="nc-stor-arch-num">0{i + 1}</span>
                <h3 className="nc-stor-arch-title">{l.title}</h3>
                <p className="nc-stor-arch-desc">{l.desc}</p>
              </div>
            ))}
          </div>
        </NbSection>

        <NbSection ref={statesRef} ariaLabel="Engine states">
          <NbSectionHeader
            monoLabel="[ENGINE STATES]"
            headline="Lifecycle of the engine."
            sub="VantaDB transitions through well-defined states for durability and crash safety."
          />

          <div className="nc-stor-state nc-stor-part">
            {ENGINE_STATES.map((s) => (
              <div key={s.state} className="nc-stor-state-card">
                <span className="nc-stor-state-label">STATE</span>
                <div className="nc-stor-state-name">{s.state}</div>
                <p className="nc-stor-state-desc">{s.desc}</p>
              </div>
            ))}
          </div>
        </NbSection>

        <NbSection variant="dark" className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="nc-stor-cta">
              <div>
                <h2 className="nc-stor-cta-title">One binary. Three engines. Zero ops.</h2>
                <p className="nc-stor-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="nc-stor-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}

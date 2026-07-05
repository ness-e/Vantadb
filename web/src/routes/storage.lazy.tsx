import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import { PendingComponent } from "@/components/PendingComponent";
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
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(stackRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, stackRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(archRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, archRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(statesRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
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

          <div className="storage-compare-grid nb-engine-part">
            <div className="storage-compare-col">
              <div className="nb-cell storage-cell-border-bottom">
                <span className="storage-label-mono storage-label-mono--steel">
                  LEGACY \u2014 3 services
                </span>
              </div>
              {LEGACY_SERVICES.map((s) => (
                <div
                  key={s.name}
                  className="nb-cell"
                  style={{ borderLeft: `3px solid ${s.color}` }}
                >
                  <div className="storage-service-name">{s.name}</div>
                  <div className="storage-service-role">{s.role}</div>
                </div>
              ))}
              <div className="nb-cell storage-cell-border-top">
                <span className="storage-label-mono storage-label-mono--danger">
                  3 SDKs \u00B7 3 bills \u00B7 $200/mo
                </span>
              </div>
            </div>

            <div className="storage-arrow-col">
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

            <div className="storage-compare-col storage-compare-col--accent">
              <div className="nb-cell storage-cell-border-bottom">
                <span className="storage-label-mono storage-label-mono--amber">
                  VantaDB \u2014 1 binary
                </span>
              </div>
              <div className="nb-cell">
                <div className="storage-brand-name">vantadb</div>
                <div className="storage-brand-tags">
                  Vector \u00B7 Full-text \u00B7 Hybrid \u00B7 WAL
                </div>
              </div>
              <div className="nb-cell storage-cell-border-top">
                <span className="storage-label-mono storage-label-mono--amber">
                  1 SDK \u00B7 1 install \u00B7 $0
                </span>
              </div>
            </div>
          </div>

          <div className="nb-grid nb-grid--cols-2 nb-engine-part">
            <div className="nb-cell">
              <span className="storage-label-mono storage-label-mono--steel">LEGACY STACK</span>
              <ul className="storage-ul-reset storage-list-gap">
                {LEGACY_STACK.map((item) => (
                  <li key={item} className="storage-li-item storage-li-item--muted">
                    <span className="storage-icon storage-icon--danger">\u2717</span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nb-cell storage-cell-amber-border">
              <span className="storage-label-mono storage-label-mono--amber">VANTADB</span>
              <ul className="storage-ul-reset storage-list-gap">
                {VANTA_STACK.map((item) => (
                  <li key={item} className="storage-li-item storage-li-item--foreground">
                    <span className="storage-icon storage-icon--vanta">\u2713</span>
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

          <div className="nb-grid nb-grid--cols-3 nb-engine-part">
            {LAYERS.map((l, i) => (
              <div key={i} className="nb-cell storage-arch-cell">
                <span className="storage-arch-num">0{i + 1}</span>
                <h3 className="storage-arch-title">{l.title}</h3>
                <p className="storage-arch-desc">{l.desc}</p>
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

          <div className="nb-grid nb-grid--cols-3 nb-engine-part">
            {ENGINE_STATES.map((s) => (
              <div key={s.state} className="nb-cell">
                <span className="storage-state-label">STATE</span>
                <div className="storage-state-name">{s.state}</div>
                <p className="storage-state-desc">{s.desc}</p>
              </div>
            ))}
          </div>
        </NbSection>

        <NbSection variant="dark" className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="storage-cta-layout">
              <div>
                <h2 className="storage-cta-title">One binary. Three engines. Zero ops.</h2>
                <p className="storage-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="storage-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}

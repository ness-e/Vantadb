import "../styles/feature-grid.css";
import { NbSection, NbSectionHeader } from "./nb";

const FEATURES = [
  {
    id: "hybrid",
    title: "Hybrid Search",
    desc: "HNSW + BM25 fused scoring. One query, ranked results.",
    cols: 5,
    rows: 2,
    featured: true,
  },
  { id: "wal", title: "Write-Ahead Log", desc: "Crash-safe durability. Zero data loss.", cols: 3 },
  { id: "zeroops", title: "Zero-Ops", desc: "No servers. No daemons. No cloud bills.", cols: 4 },
  { id: "pyo3", title: "PyO3 Native", desc: "Rust core, Python bindings. Native speed.", cols: 3 },
  {
    id: "converged",
    title: "Converged Engine",
    desc: "SQL, vectors, and full-text search — one engine.",
    cols: 6,
  },
  { id: "embed", title: "Embed Anywhere", desc: "2MB binary. One file. Any platform.", cols: 3 },
];

function NbFgIcon({ id }: { id: string }) {
  const svg = (() => {
    switch (id) {
      case "hybrid":
        return (
          <>
            <circle cx="9" cy="12" r="7" stroke="currentColor" strokeWidth="2" />
            <circle cx="15" cy="12" r="7" stroke="currentColor" strokeWidth="2" />
          </>
        );
      case "wal":
        return (
          <>
            <rect x="4" y="5" width="16" height="4" rx="1" fill="currentColor" opacity="0.3" />
            <rect x="4" y="10" width="16" height="4" rx="1" fill="currentColor" opacity="0.6" />
            <rect x="4" y="15" width="16" height="4" rx="1" fill="currentColor" />
          </>
        );
      case "zeroops":
        return (
          <path d="M12 3v5m0 0a7 7 0 1 1-7 7m7-7a7 7 0 0 1 7 7" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
        );
      case "pyo3":
        return (
          <>
            <path d="M6 9l-4 3 4 3" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M18 9l4 3-4 3" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round" />
            <line x1="14" y1="6" x2="10" y2="18" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
          </>
        );
      case "embed":
        return (
          <path d="M4 12h6m0 0l-3-3m3 3l-3 3m13-3h-6m0 0l3-3m-3 3l3 3" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round" />
        );
      default:
        return <circle cx="12" cy="12" r="4" fill="none" stroke="currentColor" strokeWidth="2" />;
    }
  })();

  return (
    <div className="nb-icon-box" aria-hidden="true">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
        {svg}
      </svg>
    </div>
  );
}

export function NbFeatureGrid() {
  return (
    <NbSection ariaLabel="Features">
      <div className="nb-split-7-5">
        <div>
          <NbSectionHeader
            monoLabel="[CAPABILITIES]"
            headline="Everything you need. Nothing you don't."
            sub="HNSW vector search, BM25 full-text, SQL — fused in one embedded engine. Sub-millisecond queries. Zero servers."
          />
        </div>
        <div>
          {FEATURES.filter((f) => f.id !== "converged").map((feat) => (
            <article key={feat.id} className="nb-fg-row">
              <NbFgIcon id={feat.id} />
              <div>
                <h3 className="nb-fg-title">{feat.title}</h3>
                <p className="nb-fg-desc">{feat.desc}</p>
              </div>
            </article>
          ))}
        </div>
      </div>
    </NbSection>
  );
}

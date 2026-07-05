import "../styles/feature-grid.css";

const FEATURES = [
  {
    id: "hybrid",
    title: "Hybrid Search",
    desc: "HNSW + BM25 fused scoring. One query, ranked results.",
    cols: 5,
    rows: 2,
    featured: true,
  },
  {
    id: "wal",
    title: "Write-Ahead Log",
    desc: "Crash-safe durability. Zero data loss.",
    cols: 3,
  },
  {
    id: "zeroops",
    title: "Zero-Ops",
    desc: "No servers. No daemons. No cloud bills.",
    cols: 4,
  },
  {
    id: "pyo3",
    title: "PyO3 Native",
    desc: "Rust core, Python bindings. Native speed.",
    cols: 3,
  },
  {
    id: "converged",
    title: "Converged Engine",
    desc: "SQL, vectors, and full-text search — one engine.",
    cols: 6,
  },
  {
    id: "embed",
    title: "Embed Anywhere",
    desc: "2MB binary. One file. Any platform.",
    cols: 3,
  },
];

export function NbFeatureGrid() {
  return (
    <section className="nb-section" aria-label="Features">
      <div className="nb-inner">
        <div className="nb-section-header nb-section-header--bordered">
          <h2 className="nb-amber-title">Features</h2>
        </div>

        <div className="nb-feature-grid">
          {FEATURES.map((feat) => (
            <article
              key={feat.id}
              className={`nb-feature-cell${feat.featured ? " nb-feature-cell--featured" : ""}`}
              style={{
                gridColumn: `span ${feat.cols}`,
                ...(feat.rows ? { gridRow: `span ${feat.rows}` } : {}),
              }}
            >
              <h3
                className={`nb-feature-title${feat.featured ? " nb-feature-title--featured" : ""}`}
              >
                {feat.title}
              </h3>
              <p className="nb-feature-desc">{feat.desc}</p>
            </article>
          ))}

          <div className="nb-feature-banner">
            <p className="nb-feature-banner-text">
              Embed anywhere — 2MB binary. One file. Any platform.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}

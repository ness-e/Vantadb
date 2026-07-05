import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import "../../styles/ai-ide-tooling.css";

export const Route = createLazyRoute("/solutions/ai-ide-tooling")({
  component: IdeToolingPage,
});

const USE_CASES = [
  {
    num: "01",
    title: "Semantic Symbol Lookup",
    desc: '"Where do we validate tokens?" returns the auth middleware function — even if "token" appears in zero comments.',
  },
  {
    num: "02",
    title: "Pattern Matching",
    desc: '"Find all places we fetch from an API and cache the result" — understands the architectural pattern, not just code.',
  },
  {
    num: "03",
    title: "Context Retrieval",
    desc: "When editing a file, automatically surface related functions, type definitions, and usage examples from across the codebase.",
  },
];

const PROBLEMS = [
  "grep/ripgrep: lexical only, no semantic understanding",
  "IDE symbol search: requires indexed projects, misses patterns",
  "GitHub Code Search: cloud-dependent, can't search local repos",
  "Cloud vector DBs: add latency, require network, leak code context",
];

const BENEFITS = [
  '"Find the function that handles JWT authentication" — not just keywords',
  "AST-aware indexing: functions, classes, imports as structured metadata",
  "BM25 for symbol search + HNSW for semantic = hybrid retrieval",
  "Runs in your IDE extension process — no cloud, no latency",
];

function IdeToolingPage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="03"
        title={
          <span>
            Your codebase,
            <br />
            searchable by meaning.
          </span>
        }
        sub="Augment your IDE with semantic code search, AST-aware retrieval, and context-aware completions. VantaDB powers the next generation of AI coding tools."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <h2 className="ai-ide-tooling-section-title">The Gap</h2>
          <div className="nb-divider" />

          <div className="ai-ide-tooling-grid-2col">
            <div className="nb-cell ai-ide-tooling-cell-padded">
              <span className="ai-ide-tooling-label-muted">Code search is still text-only</span>
              <ul className="nb-list">
                {PROBLEMS.map((p) => (
                  <li key={p} className="ai-ide-tooling-list-item-muted">
                    <span className="ai-ide-tooling-list-icon-danger">✗</span>
                    {p}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nb-cell ai-ide-tooling-cell-amber-border">
              <span className="ai-ide-tooling-label-amber">Semantic, embedded</span>
              <ul className="nb-list">
                {BENEFITS.map((s) => (
                  <li key={s} className="ai-ide-tooling-list-item-foreground">
                    <span className="ai-ide-tooling-list-icon-amber">✓</span>
                    {s}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <h2 className="ai-ide-tooling-section-title">Use Cases</h2>
          <div className="nb-divider" />
          <p className="ai-ide-tooling-title-large">Beyond grep.</p>

          <div className="nb-grid nb-grid--cols-3">
            {USE_CASES.map((uc) => (
              <div key={uc.num} className="nb-cell ai-ide-tooling-cell-padded">
                <span className="ai-ide-tooling-card-num">{uc.num}</span>
                <h3 className="ai-ide-tooling-card-title">{uc.title}</h3>
                <p className="ai-ide-tooling-card-desc">{uc.desc}</p>
              </div>
            ))}
          </div>

          <div className="nb-card ai-ide-tooling-card-surface">
            <div className="ai-ide-tooling-inner-grid">
              <span className="ai-ide-tooling-inner-label">HOW IT WORKS</span>
              <p className="ai-ide-tooling-inner-text">
                Each code unit (function, class, module) is indexed as a vector embedding plus
                structured AST metadata (name, signature, dependencies, docstring). Queries use
                hybrid search: BM25 for symbol matching, HNSW for semantic similarity, with RRF
                fusion for final ranking.
              </p>
            </div>
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-block-amber ai-ide-tooling-cta-block">
            <span className="ai-ide-tooling-cta-label">BUILD IDE TOOLS</span>
            <p className="ai-ide-tooling-cta-text">
              Read the docs to integrate semantic code search.
            </p>
            <Link to="/docs" className="nb-btn nb-btn--ghost ai-ide-tooling-cta-link">
              DOCS
            </Link>
          </div>
        </div>
      </section>

      <style>{`
        @media (max-width: 640px) {
          [style*="grid-template-columns: 120px 1fr"] { grid-template-columns: 1fr !important; gap: var(--space-sm) !important; }
        }
      `}</style>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="ai-ide-tooling-pending">
      <span className="ai-ide-tooling-pending-text">Loading...</span>
    </div>
  );
}

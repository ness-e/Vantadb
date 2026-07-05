import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";

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
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-title)",
              fontWeight: 700,
              color: "var(--foreground)",
              margin: "0 0 var(--space-md)",
            }}
          >
            The Gap
          </h2>
          <div className="nb-divider" />

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: "1px",
              background: "var(--border-visible)",
              marginTop: "var(--space-xl)",
            }}
          >
            <div className="nb-cell" style={{ padding: "var(--space-xl)" }}>
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "var(--text-micro)",
                  color: "var(--muted)",
                  marginBottom: "var(--space-md)",
                  display: "block",
                }}
              >
                Code search is still text-only
              </span>
              <ul className="nb-list">
                {PROBLEMS.map((p) => (
                  <li key={p} style={{ color: "var(--muted)" }}>
                    <span
                      style={{
                        color: "var(--danger)",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
                        marginRight: "var(--space-2xs)",
                      }}
                    >
                      ✗
                    </span>
                    {p}
                  </li>
                ))}
              </ul>
            </div>
            <div
              className="nb-cell"
              style={{
                padding: "var(--space-xl)",
                borderLeft: "2px solid var(--amber)",
                background: "var(--surface-alt)",
              }}
            >
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "var(--text-micro)",
                  color: "var(--amber)",
                  fontWeight: 700,
                  marginBottom: "var(--space-md)",
                  display: "block",
                }}
              >
                Semantic, embedded
              </span>
              <ul className="nb-list">
                {BENEFITS.map((s) => (
                  <li key={s} style={{ color: "var(--foreground)" }}>
                    <span
                      style={{
                        color: "var(--amber)",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
                        marginRight: "var(--space-2xs)",
                      }}
                    >
                      ✓
                    </span>
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
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-title)",
              fontWeight: 700,
              color: "var(--foreground)",
              margin: "0 0 var(--space-md)",
            }}
          >
            Use Cases
          </h2>
          <div className="nb-divider" />
          <p
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-display)",
              fontWeight: 800,
              letterSpacing: "var(--tracking-display)",
              margin: "var(--space-sm) 0 var(--space-xl)",
              lineHeight: 1.05,
            }}
          >
            Beyond grep.
          </p>

          <div className="nb-grid nb-grid--cols-3">
            {USE_CASES.map((uc) => (
              <div key={uc.num} className="nb-cell" style={{ padding: "var(--space-xl)" }}>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-micro)",
                    color: "var(--amber)",
                    fontWeight: 700,
                    marginBottom: "var(--space-2xs)",
                    display: "block",
                  }}
                >
                  {uc.num}
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "var(--text-title)",
                    fontWeight: 700,
                    letterSpacing: "var(--tracking-display)",
                    color: "var(--foreground)",
                    margin: "0 0 var(--space-2xs)",
                  }}
                >
                  {uc.title}
                </h3>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "var(--text-code)",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    margin: 0,
                  }}
                >
                  {uc.desc}
                </p>
              </div>
            ))}
          </div>

          <div className="nb-card" style={{ marginTop: "1px", background: "var(--surface)" }}>
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "120px 1fr",
                gap: "var(--space-lg)",
                alignItems: "start",
              }}
            >
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "var(--text-micro)",
                  color: "var(--muted)",
                  marginBottom: 0,
                }}
              >
                HOW IT WORKS
              </span>
              <p
                style={{
                  fontFamily: "var(--font-sans)",
                  fontSize: "var(--text-code)",
                  color: "var(--muted)",
                  lineHeight: 1.7,
                  margin: 0,
                }}
              >
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
          <div className="nb-block-amber" style={{ textAlign: "center" }}>
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-micro)",
                color: "var(--text-on-amber)",
                marginBottom: "var(--space-2xs)",
                display: "block",
              }}
            >
              BUILD IDE TOOLS
            </span>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "var(--text-body)",
                color: "var(--text-on-amber)",
                margin: "var(--space-2xs) 0",
                opacity: 0.85,
              }}
            >
              Read the docs to integrate semantic code search.
            </p>
            <Link
              to="/docs"
              className="nb-btn nb-btn--ghost"
              style={{
                borderColor: "var(--text-on-amber)",
                color: "var(--text-on-amber)",
                boxShadow: "var(--shadow-brutal)",
              }}
            >
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
    <div
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        height: "60vh",
        color: "var(--muted)",
      }}
    >
      <span
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "var(--text-label)",
          color: "var(--muted)",
          marginBottom: 0,
        }}
      >
        Loading...
      </span>
    </div>
  );
}

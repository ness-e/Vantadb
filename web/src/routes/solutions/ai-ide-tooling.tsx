import { createFileRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/solutions/ai-ide-tooling")({
  head: () => ({
    meta: [
      { title: "VantaDB — AI-Powered IDE Tooling" },
      {
        name: "description",
        content:
          "Augment your coding workflow with semantic code search, AST-aware retrieval, and context-aware completions powered by an embedded vector database.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/solutions/ai-ide-tooling" }],
  }),
  component: IdeToolingPage,
});

// ── Data ─────────────────────────────────────────────────────────────────────
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
    <div className="engine-page">
      <SwissSubpageHero
        num="03"
        eyebrow="Solution — AI IDE Tooling"
        title={
          <span>
            Your codebase,
            <br />
            searchable by meaning.
          </span>
        }
        sub="Augment your IDE with semantic code search, AST-aware retrieval, and context-aware completions. VantaDB powers the next generation of AI coding tools."
      />

      <main className="engine-main">
        {/* Section 1: Problem vs Solution */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 02 — The Gap</span>
          <div
            className="swiss-grid-12"
            style={{ alignItems: "start", marginTop: "3rem", gap: "1px" }}
          >
            <div
              className="col-span-6"
              style={{ border: "1px solid var(--border)", padding: "2.5rem" }}
            >
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "1.1rem",
                  fontWeight: 800,
                  letterSpacing: "-0.03em",
                  color: "var(--steel)",
                  marginBottom: "2rem",
                  textTransform: "uppercase",
                }}
              >
                Code search is still text-only
              </h2>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "1rem",
                }}
              >
                {PROBLEMS.map((p, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.82rem",
                      color: "var(--muted)",
                      lineHeight: 1.5,
                    }}
                  >
                    <span
                      style={{
                        color: "#ff3b30",
                        fontWeight: 700,
                        minWidth: "1rem",
                        fontFamily: "var(--font-mono)",
                        flexShrink: 0,
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
              className="col-span-6"
              style={{
                border: "1px solid var(--border)",
                borderLeft: "2px solid var(--amber)",
                padding: "2.5rem",
                background: "var(--surface)",
              }}
            >
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "1.1rem",
                  fontWeight: 800,
                  letterSpacing: "-0.03em",
                  color: "var(--amber)",
                  marginBottom: "2rem",
                  textTransform: "uppercase",
                }}
              >
                Semantic, embedded
              </h2>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "1rem",
                }}
              >
                {BENEFITS.map((s, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.82rem",
                      color: "var(--foreground)",
                      lineHeight: 1.5,
                    }}
                  >
                    <span
                      style={{
                        color: "var(--amber)",
                        fontWeight: 700,
                        minWidth: "1rem",
                        fontFamily: "var(--font-mono)",
                        flexShrink: 0,
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
        </section>

        {/* Section 2: Use Cases */}
        <section className="engine-section">
          <span className="swiss-eyebrow">02 / 02 — Use Cases</span>
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "clamp(1.5rem, 3vw, 2.5rem)",
              fontWeight: 800,
              letterSpacing: "-0.04em",
              margin: "1.25rem 0 3rem",
              lineHeight: 1.05,
            }}
          >
            Beyond grep.
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
            {USE_CASES.map((uc) => (
              <div
                key={uc.num}
                style={{
                  background: "var(--background)",
                  padding: "2.5rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.75rem",
                  transition: "background-color 150ms var(--ease-cut)",
                }}
                onMouseEnter={(e) =>
                  ((e.currentTarget as HTMLDivElement).style.background = "var(--surface-raised)")
                }
                onMouseLeave={(e) =>
                  ((e.currentTarget as HTMLDivElement).style.background = "var(--background)")
                }
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
                  {uc.num}
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
                  {uc.title}
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
                  {uc.desc}
                </p>
              </div>
            ))}
          </div>

          {/* Architecture note */}
          <div
            style={{
              marginTop: "1px",
              border: "1px solid var(--border)",
              background: "var(--surface)",
              padding: "2rem 2.5rem",
              display: "grid",
              gridTemplateColumns: "120px 1fr",
              gap: "2rem",
              alignItems: "start",
            }}
          >
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "0.6rem",
                color: "var(--steel)",
                textTransform: "uppercase",
                letterSpacing: "0.08em",
              }}
            >
              HOW IT WORKS
            </span>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "0.82rem",
                color: "var(--muted)",
                lineHeight: 1.7,
                margin: 0,
              }}
            >
              Each code unit (function, class, module) is indexed as a vector embedding plus
              structured AST metadata (name, signature, dependencies, docstring). Queries use hybrid
              search: BM25 for symbol matching, HNSW for semantic similarity, with RRF fusion for
              final ranking.
            </p>
          </div>
        </section>
      </main>
    </div>
  );
}

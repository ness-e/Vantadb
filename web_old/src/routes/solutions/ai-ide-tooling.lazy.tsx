import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbBlockAmber, NbSection, NbSectionHeader } from "@/components/nb";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
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
  const gapRef = useRef<HTMLElement>(null);
  const useCasesRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gapRef.current?.querySelectorAll<HTMLElement>(".nc-ai-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, gapRef);

  useAnimationSafe(() => {
    const parts = useCasesRef.current?.querySelectorAll<HTMLElement>(".nc-ai-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, useCasesRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p24"
        title={
          <span>
            Your codebase,
            <br />
            searchable by meaning.
          </span>
        }
        sub="Augment your IDE with semantic code search, AST-aware retrieval, and context-aware completions. VantaDB powers the next generation of AI coding tools."
      />

      <NbSection ref={gapRef} ariaLabel="The Gap">
        <NbSectionHeader
          monoLabel="[THE GAP]"
          headline="Code search is still text-only."
          sub="Traditional tools search strings, not meaning. VantaDB embeds semantic understanding directly into your IDE tooling."
        />

        <div className="nc-ai-part">
          <div className="nc-ai-compare">
            <div className="nc-ai-col">
              <span className="nc-ai-col-label nc-ai-col-label--steel">Problems</span>
              <ul className="nc-ai-list">
                {PROBLEMS.map((p) => (
                  <li key={p} className="nc-ai-item">
                    <span className="nc-ai-icon nc-ai-icon--danger">✗</span>
                    {p}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nc-ai-col nc-ai-col--amber">
              <span className="nc-ai-col-label nc-ai-col-label--amber">Solution</span>
              <ul className="nc-ai-list">
                {BENEFITS.map((s) => (
                  <li key={s} className="nc-ai-item nc-ai-item--fg">
                    <span className="nc-ai-icon nc-ai-icon--amber">✓</span>
                    {s}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </NbSection>

      <NbSection ref={useCasesRef} className="nb-bg-cross--faint" ariaLabel="Use Cases">
        <NbSectionHeader
          monoLabel="[USE CASES]"
          headline="Beyond grep."
          sub="Natural-language queries that understand your codebase architecture."
        />

        <div className="nc-ai-part">
          <div className="nc-ai-usecases">
            {USE_CASES.map((uc) => (
              <div key={uc.num} className="nc-ai-ucard">
                <span className="nc-ai-unum">{uc.num}</span>
                <h3 className="nc-ai-utitle">{uc.title}</h3>
                <p className="nc-ai-udesc">{uc.desc}</p>
              </div>
            ))}
          </div>
        </div>

        <div className="nc-ai-part">
          <div className="nc-ai-banner">
            <span className="nb-mono-label">HYBRID SEARCH</span>
            <p className="nc-ai-banner-text">
              Each code unit (function, class, module) is indexed as a vector embedding plus
              structured AST metadata (name, signature, dependencies, docstring). Queries use hybrid
              search: BM25 for symbol matching, HNSW for semantic similarity, with RRF fusion for
              final ranking.
            </p>
          </div>
        </div>
      </NbSection>

      <NbSection ariaLabel="Get started">
        <NbBlockAmber>
          <div className="nb-text-center">
            <span className="nb-mono-label">BUILD IDE TOOLS</span>
            <p className="nb-section-sub">Read the docs to integrate semantic code search.</p>
            <Link to="/docs" className="nb-btn nb-btn--ghost">
              DOCS
            </Link>
          </div>
        </NbBlockAmber>
      </NbSection>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="ai-ide-tooling-pending">
      <span>Loading...</span>
    </div>
  );
}

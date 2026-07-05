import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
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
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(gapRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, gapRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(useCasesRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
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

        <div className="nb-engine-part">
          <div className="ai-ide-tooling-grid-2col">
            <div className="ai-ide-tooling-cell-padded">
              <span className="nb-mono-label">Problems</span>
              <ul className="nb-list">
                {PROBLEMS.map((p) => (
                  <li key={p} className="ai-ide-tooling-list-item-muted">
                    <span className="ai-ide-tooling-list-icon ai-ide-tooling-list-icon--danger">
                      ✗
                    </span>
                    {p}
                  </li>
                ))}
              </ul>
            </div>
            <div className="ai-ide-tooling-cell-amber-border">
              <span className="nb-mono-label">Solution</span>
              <ul className="nb-list">
                {BENEFITS.map((s) => (
                  <li key={s} className="ai-ide-tooling-list-item-foreground">
                    <span className="ai-ide-tooling-list-icon ai-ide-tooling-list-icon--amber">
                      ✓
                    </span>
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

        <div className="nb-engine-part">
          <div className="nb-grid nb-grid--cols-3">
            {USE_CASES.map((uc) => (
              <div key={uc.num} className="nb-cell ai-ide-tooling-cell-padded">
                <span className="nb-mono-label">{uc.num}</span>
                <h3 className="nb-card-frame-title">{uc.title}</h3>
                <p className="nb-card-frame-desc">{uc.desc}</p>
              </div>
            ))}
          </div>
        </div>

        <div className="nb-engine-part">
          <div className="ai-ide-tooling-inner-grid">
            <span className="nb-mono-label">HYBRID SEARCH</span>
            <p className="ai-ide-tooling-inner-text">
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
            <p className="ai-ide-tooling-cta-text">
              Read the docs to integrate semantic code search.
            </p>
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

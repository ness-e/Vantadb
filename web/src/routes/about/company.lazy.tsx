import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import "../../styles/about-company.css";

export const Route = createLazyRoute("/about/company")({
  component: CompanyPage,
});

const VALUES = [
  {
    num: "01",
    title: "Radical Simplicity",
    desc: "One binary, one pip install, zero servers. Complexity is the enemy — we eat it so developers don't have to.",
  },
  {
    num: "02",
    title: "Performance Without Compromise",
    desc: "1.2ms p50 queries at 0.998 Recall@10. Every microsecond matters when your agent is waiting.",
  },
  {
    num: "03",
    title: "Developer Empathy First",
    desc: "We ship SDKs, docs, and APIs that feel like they were built by developers for developers — because they were.",
  },
  {
    num: "04",
    title: "Open by Default",
    desc: "Open core, open benchmarks, open roadmap. Our community trusts us because we show receipts.",
  },
];

const COMPARISON_LEFT = [
  "Pinecone: $70/mo + per-vector pricing",
  "Weaviate/Qdrant: server process + ops team",
  "LanceDB: limited hybrid search",
  "LanceDB: data model is a second thought",
];

const COMPARISON_RIGHT = [
  "VantaDB: one binary, zero ops, Apache 2.0 license",
  "HNSW + BM25 + hybrid in a single query",
  "Embedded in your process — no network hop",
  "Sub-millisecond hybrid search",
];

function CompanyPage() {
  const purposeRef = useRef<HTMLElement>(null);
  const valuesRef = useRef<HTMLElement>(null);
  const compareRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-ac-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(purposeRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, purposeRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-ac-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(valuesRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, valuesRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-ac-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(compareRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, compareRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p20"
        title={
          <span>
            Built for the
            <br />
            AI-native era.
          </span>
        }
        sub="VantaDB unifies vector search (HNSW), lexical search (BM25), and hybrid search (RRF) in a single Rust binary. Zero servers. Zero ops. One pip install."
      />

      <NbSection ref={purposeRef} ariaLabel="Purpose">
        <NbSectionHeader
          monoLabel="[PURPOSE]"
          headline="Make vector-native data infrastructure invisible."
          sub="Every AI agent, every RAG pipeline, every intelligent application deserves a database that embeds in-process but understands vectors, text, and hybrid search — without requiring a dedicated infrastructure team."
        />
      </NbSection>

      <NbSection ref={valuesRef} className="nb-bg-cross--faint" ariaLabel="Values">
        <NbSectionHeader
          monoLabel="[VALUES]"
          headline="What drives us."
          sub="Four principles that guide every line of code."
        />

        <div className="nc-ac-part">
          <div className="nc-ac-values">
            {VALUES.map((v) => (
              <div key={v.num} className="nc-ac-value-card">
                <span className="nb-mono-label">{v.num}</span>
                <h3 className="nb-card-frame-title">{v.title}</h3>
                <p className="nb-card-frame-desc">{v.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </NbSection>

      <NbSection ref={compareRef} ariaLabel="Why VantaDB">
        <NbSectionHeader
          monoLabel="[WHY VANTADB]"
          headline="The AI stack shouldn't need a database team."
          sub="Compare the alternatives."
        />

        <div className="nc-ac-part">
          <div className="nc-ac-compare">
            <div className="nc-ac-compare-left">
              <span className="nc-ac-compare-label">The alternatives</span>
              <ul className="nc-ac-compare-list">
                {COMPARISON_LEFT.map((item) => (
                  <li key={item} className="nc-ac-compare-item nc-ac-compare-item--muted">
                    <span className="nc-ac-compare-icon">✗</span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nc-ac-compare-right">
              <span className="nc-ac-compare-label">VantaDB</span>
              <ul className="nc-ac-compare-list">
                {COMPARISON_RIGHT.map((item) => (
                  <li key={item} className="nc-ac-compare-item nc-ac-compare-item--fg">
                    <span className="nc-ac-compare-icon">✓</span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </NbSection>

      <NbSection ariaLabel="Get started">
        <NbBlockAmber>
          <div className="nb-text-center">
            <span className="nb-mono-label">READ OUR STORY</span>
            <p className="about-company-cta-desc">Learn more about our community.</p>
            <Link to="/about/community" className="nb-btn nb-btn--ghost">
              COMMUNITY
            </Link>
          </div>
        </NbBlockAmber>
      </NbSection>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="about-company-pending">
      <span>Loading...</span>
    </div>
  );
}

import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbBlockAmber, NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import "../styles/why-vantadb.css";

export const Route = createLazyRoute("/why-vantadb")({
  component: WhyVantaDBPage,
  pendingComponent: PendingComponent,
});

const COMPARISONS = [
  {
    category: "Architecture",
    vantadb: "Embedded, zero-infrastructure",
    others: "Requires servers, network calls",
  },
  { category: "Latency", vantadb: "<1ms p50 local", others: "3–50ms network round-trip" },
  {
    category: "Embedding",
    vantadb: "Any provider (OpenAI, Ollama, LiteLLM)",
    others: "Vendor-locked models",
  },
  { category: "Storage", vantadb: "Single binary file, 2 MB", others: "Database cluster required" },
  {
    category: "Search",
    vantadb: "HNSW + BM25 hybrid fused",
    others: "Vector-only or separate text search",
  },
  { category: "License", vantadb: "Apache 2.0", others: "BSL or proprietary" },
];

const PRINCIPLES = [
  {
    title: "Library, Not a Service",
    desc: "Embed VantaDB like SQLite. No daemons, no containers, no cloud bills.",
  },
  {
    title: "Hybrid by Default",
    desc: "HNSW + BM25 fused scoring. One query, ranked results, zero glue code.",
  },
  {
    title: "Built for AI Agents",
    desc: "Persistent memory, tool-native access, sub-millisecond recall. Your agent's hippocampus.",
  },
];

function WhyVantaDBPage() {
  const comparisonRef = useRef<HTMLElement>(null);
  const principlesRef = useRef<HTMLElement>(null);
  const ctaRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = comparisonRef.current?.querySelectorAll<HTMLElement>(".nc-why-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, comparisonRef);

  useAnimationSafe(() => {
    const parts = principlesRef.current?.querySelectorAll<HTMLElement>(".nc-why-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, principlesRef);

  useAnimationSafe(() => {
    const parts = ctaRef.current?.querySelectorAll<HTMLElement>(".nc-why-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, ctaRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p15"
        title="Why VantaDB"
        sub="21 reasons to ship embedded vector search."
      />

      <main>
        <NbSection ref={comparisonRef} ariaLabel="Comparison">
          <NbSectionHeader
            monoLabel="[COMPARISON]"
            headline="VantaDB vs the field."
            sub="Side-by-side comparison of architecture, latency, embedding, storage, search capability, and licensing."
          />
          <table className="nc-why-table nc-why-part">
            <tbody>
              {COMPARISONS.map((c) => (
                <tr key={c.category}>
                  <td className="nc-why-cat">{c.category}</td>
                  <td className="nc-why-vanta">{c.vantadb}</td>
                  <td className="nc-why-others">{c.others}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </NbSection>

        <NbSection
          ref={principlesRef}
          variant="lg"
          className="nb-bg-cross--faint"
          ariaLabel="Principles"
        >
          <NbSectionHeader
            monoLabel="[PRINCIPLES]"
            headline="Design philosophy."
            sub="Three axioms that guide every decision in VantaDB."
          />
          <div className="nc-why-pillars nc-why-part">
            {PRINCIPLES.map((p) => (
              <div key={p.title} className="nc-why-pillar nc-why-pillar--amber">
                <h3 className="nc-why-pillar-title">{p.title}</h3>
                <p className="nc-why-pillar-desc">{p.desc}</p>
              </div>
            ))}
          </div>
        </NbSection>

        <NbSection ref={ctaRef} ariaLabel="Call to action">
          <div className="nc-why-part">
            <NbBlockAmber as="div">
              <div className="nc-why-cta">
                <div>
                  <h2 className="nc-why-cta-heading">Ready to ship.</h2>
                  <p className="nc-why-cta-sub">One dependency. Zero infrastructure.</p>
                </div>
                <Link to="/about/company" className="nb-btn nb-btn--ghost">
                  cd about/company
                </Link>
              </div>
            </NbBlockAmber>
          </div>
        </NbSection>
      </main>
    </div>
  );
}

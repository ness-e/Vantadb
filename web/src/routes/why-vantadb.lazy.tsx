import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import { PendingComponent } from "@/components/PendingComponent";
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
  { category: "Embedding", vantadb: "Any ONNX model", others: "Vendor-locked models" },
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
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(comparisonRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, comparisonRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(principlesRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, principlesRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(ctaRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, ctaRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p15"
        title="Why VantaDB"
        sub="21 reasons to ship embedded vector search."
      />

      <NbSection ref={comparisonRef} ariaLabel="Comparison">
        <NbSectionHeader
          monoLabel="[COMPARISON]"
          headline="VantaDB vs the field."
          sub="Side-by-side comparison of architecture, latency, embedding, storage, search capability, and licensing."
        />
        <div className="nb-grid nb-grid--cols-2 why-vantadb-compare-grid nb-engine-part">
          {COMPARISONS.map((c) => (
            <div key={c.category} className="why-vantadb-compare-item">
              <span className="nb-mono-label">{c.category}</span>
              <div className="why-vantadb-compare-row">
                <span className="why-vantadb-compare-vantadb">{c.vantadb}</span>
                <span className="why-vantadb-arrow">→</span>
                <span className="why-vantadb-compare-others">{c.others}</span>
              </div>
            </div>
          ))}
        </div>
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
        <div className="nb-grid nb-grid--cols-3 nb-engine-part">
          {PRINCIPLES.map((p) => (
            <div key={p.title} className="why-vantadb-principle-card">
              <h3 className="nb-card-frame-title">{p.title}</h3>
              <p className="nb-card-frame-desc">{p.desc}</p>
            </div>
          ))}
        </div>
      </NbSection>

      <NbSection ref={ctaRef} ariaLabel="Call to action">
        <div className="nb-engine-part">
          <NbBlockAmber as="div">
            <div className="why-vantadb-cta-row">
              <div>
                <h2 className="why-vantadb-cta-heading">Ready to ship.</h2>
                <p className="why-vantadb-cta-sub">One dependency. Zero infrastructure.</p>
              </div>
              <Link to="/about/company" className="nb-btn nb-btn--ghost">
                cd about/company
              </Link>
            </div>
          </NbBlockAmber>
        </div>
      </NbSection>
    </div>
  );
}

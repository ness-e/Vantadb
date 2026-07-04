import { useRef, useState } from "react";
import { gsap, useGSAP } from "../lib/gsap";

const CASES = [
  {
    id: "01",
    title: "AI Agent Memory",
    desc: "Persistent context windows for distributed agent swarms without network overhead. Local execution means zero API latency.",
    size: "large",
  },
  {
    id: "02",
    title: "Local RAG Pipeline",
    desc: "Complete semantic search on secure enterprise hardware. Air-gapped environments fully supported out of the box.",
    size: "small",
  },
  {
    id: "03",
    title: "IDE Code Intelligence",
    desc: "Embed cognitive memory into desktop applications and IDE plugins. Blazing fast code-search entirely in-process.",
    size: "small",
  },
  {
    id: "04",
    title: "Offline Knowledge Base",
    desc: "Edge-deployed semantic search without internet. Perfect for field devices, kiosks, and disconnected environments.",
    size: "wide",
  },
];

export function SwissUseCases() {
  const sectionRef = useRef<HTMLElement>(null);
  const [hoveredCard, setHoveredCard] = useState<string | null>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        gsap.fromTo(
          ".swiss-uc-card",
          { clipPath: "inset(0 100% 0 0)", opacity: 0 },
          {
            clipPath: "inset(0)",
            opacity: 1,
            duration: 0.35,
            stagger: 0.08,
            ease: "cubic-bezier(0.25, 1, 0.5, 1)",
            scrollTrigger: {
              trigger: sectionRef.current,
              start: "top 70%",
            },
          },
        );
      });
    },
    { scope: sectionRef },
  );

  return (
    <section
      ref={sectionRef}
      className="swiss-section swiss-uc-section swiss-uc-section--light"
      aria-label="Use cases"
    >
      <div className="swiss-inner">
        <header className="swiss-uc-header">
          <span className="swiss-section-label">[USE CASES]</span>
          <h2 className="swiss-uc-heading">Built for real AI workflows.</h2>
        </header>

        <div className="swiss-uc-bento">
          {CASES.map((uc) => (
            <article
              key={uc.id}
              className={`swiss-uc-card swiss-uc-card--${uc.size}${hoveredCard === uc.id ? " swiss-uc-card--hover" : ""}`}
              onMouseEnter={() => setHoveredCard(uc.id)}
              onMouseLeave={() => setHoveredCard(null)}
              aria-label={`Use case ${uc.id}: ${uc.title}`}
            >
              <span className="swiss-uc-num" aria-hidden="true">
                [{uc.id}]
              </span>
              <h3 className="swiss-uc-title">{uc.title}</h3>
              <p className="swiss-uc-desc">{uc.desc}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

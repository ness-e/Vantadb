import { useRef } from "react";
import { gsap, useGSAP } from "../lib/gsap";
import "../styles/use-cases.css";

const CASES = [
  {
    id: "01",
    title: "AI Agent Memory",
    desc: "Persistent context windows for distributed agent swarms without network overhead. Local execution means zero API latency.",
  },
  {
    id: "02",
    title: "Local RAG Pipeline",
    desc: "Complete semantic search on secure enterprise hardware. Air-gapped environments fully supported out of the box.",
  },
  {
    id: "03",
    title: "IDE Code Intelligence",
    desc: "Embed cognitive memory into desktop applications and IDE plugins. Blazing fast code-search entirely in-process.",
  },
  {
    id: "04",
    title: "Offline Knowledge Base",
    desc: "Edge-deployed semantic search without internet. Perfect for field devices, kiosks, and disconnected environments.",
  },
];

export function SwissUseCases() {
  const sectionRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const cards = gsap.utils.toArray<HTMLElement>(".usecase-card");
        if (!cards.length) return;

        gsap.fromTo(
          cards,
          { opacity: 0, y: 24 },
          {
            opacity: 1,
            y: 0,
            duration: 0.35,
            stagger: 0.08,
            ease: "cubic-bezier(0.05, 0.95, 0.3, 1)",
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
      className="nb-section"
      aria-label="Use cases"
    >
      <div className="nb-inner">
        <div className="nb-section-header">
          <span className="nb-label nb-label--amber">[USE CASES]</span>
          <h2 className="usecase-heading">
            Built for real AI workflows.
          </h2>
        </div>

        <div className="usecase-grid">
          {CASES.map((uc) => (
            <article
              key={uc.id}
              className="usecase-card"
              aria-label={`Use case ${uc.id}: ${uc.title}`}
            >
              <span className="usecase-prefix" aria-hidden="true">
                &gt; [{uc.id}]
              </span>
              <h3 className="usecase-title">{uc.title}</h3>
              <p className="usecase-desc">{uc.desc}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

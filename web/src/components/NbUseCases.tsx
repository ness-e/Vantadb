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

export function NbUseCases() {
  const sectionRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const cards = gsap.utils.toArray<HTMLElement>(".nb-uc-card");
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
    <section ref={sectionRef} className="nb-section nb-section--lg" aria-label="Use cases">
      <div className="nb-inner">
        <span className="nb-mono-label">[USE CASES]</span>
        <h2 className="nb-section-headline">Built for real AI workflows.</h2>

        <div className="nb-uc-grid">
          {CASES.map((uc) => (
            <article key={uc.id} className="nb-uc-card">
              <span className="nb-num-marker nb-num-marker--amber">{uc.id}</span>
              <div>
                <h3 className="nb-uc-title">{uc.title}</h3>
                <p className="nb-uc-desc">{uc.desc}</p>
                <span className="nb-arrow">Learn more</span>
              </div>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

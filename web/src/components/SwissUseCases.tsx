import { useRef, useState } from "react";
import { gsap, useGSAP } from "../lib/gsap";

const CASES = [
  {
    id: "01",
    title: "Multi-Agent System Memory",
    industry: "AI INFRASTRUCTURE",
    desc: "Provides instant, isolated context windows for distributed agent swarms without network overhead. Local execution means zero API latency.",
    stack: "Python • LangChain • VantaDB",
  },
  {
    id: "02",
    title: "Offline Local RAG",
    industry: "ENTERPRISE PRIVACY",
    desc: "Complete semantic search capabilities running directly on secure enterprise hardware. Air-gapped environments are fully supported out of the box.",
    stack: "Llama.cpp • FastAPI • VantaDB",
  },
  {
    id: "03",
    title: "IDE Assistants & Tooling",
    industry: "DEVELOPER TOOLS",
    desc: "Embed cognitive memory into desktop applications and IDE plugins. Blazing fast code-search running entirely in-process.",
    stack: "Electron/Tauri • Rust • VantaDB",
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
    <section ref={sectionRef} className="swiss-section swiss-uc-section" aria-label="Use cases">
      <div className="swiss-inner">
        <header className="uc-header">
          <h2 className="uc-heading">
            Applied Use Cases.
          </h2>
        </header>

        <div className="uc-list">
          {CASES.map((uc) => (
            <article
              key={uc.id}
              className={`swiss-uc-card${hoveredCard === uc.id ? " swiss-uc-card--hover" : ""}`}
              onMouseEnter={() => setHoveredCard(uc.id)}
              onMouseLeave={() => setHoveredCard(null)}
              aria-label={`${uc.industry}: ${uc.title}`}
            >
              <header>
                <span className="swiss-uc-num" aria-hidden="true">
                  {uc.id}
                </span>
              </header>

              <div className="uc-content">
                <span className="uc-industry">
                  [{uc.industry}]
                </span>

                <h3 className="uc-title">
                  {uc.title}
                </h3>

                <p className="uc-desc">
                  {uc.desc}
                </p>

                <footer className="uc-stack-wrap">
                  <span className="uc-stack">
                    {uc.stack}
                  </span>
                </footer>
              </div>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

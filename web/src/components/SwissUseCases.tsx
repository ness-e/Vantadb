import { useRef } from "react";
import { gsap, useGSAP, ScrollTrigger } from "../lib/gsap";

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

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        gsap.fromTo(
          ".swiss-uc-card",
          { opacity: 0, y: 12 },
          {
            opacity: 1,
            y: 0,
            duration: 0.5,
            stagger: 0.06,
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
      className="swiss-section swiss-uc-section"
    >
      <div className="swiss-inner">
        <div className="uc-header">
          <h2 className="uc-heading">
            Applied Use Cases.
          </h2>
        </div>

        <div className="uc-list">
          {CASES.map((uc) => (
            <div
              key={uc.id}
              className="swiss-uc-card"
              onMouseEnter={(e) => {
                const num = e.currentTarget.querySelector(".swiss-uc-num");
                if (num) {
                  (num as HTMLElement).style.color = "var(--amber)";
                }
              }}
              onMouseLeave={(e) => {
                const num = e.currentTarget.querySelector(".swiss-uc-num");
                if (num) {
                  (num as HTMLElement).style.color = "var(--subtle)";
                }
              }}
            >
              {/* Columna Izquierda: Índice numérico grande */}
              <div>
                <span className="swiss-uc-num">
                  {uc.id}
                </span>
              </div>

              {/* Columna Derecha: Contenido */}
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

                <div className="uc-stack-wrap">
                  <span className="uc-stack">
                    {uc.stack}
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

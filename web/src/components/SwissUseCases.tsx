import { useRef } from "react";
import { gsap, useGSAP, ScrollTrigger } from "../lib/gsap";

const CASES = [
  {
    id: "01",
    title: "Multi-Agent System Memory",
    industry: "AI INFRASTRUCTURE",
    desc: "Provides instant, isolated context windows for distributed agent swarms without network overhead. Local execution means zero API latency.",
    stack: "Python • LangChain • VantaDB"
  },
  {
    id: "02",
    title: "Offline Local RAG",
    industry: "ENTERPRISE PRIVACY",
    desc: "Complete semantic search capabilities running directly on secure enterprise hardware. Air-gapped environments are fully supported out of the box.",
    stack: "Llama.cpp • FastAPI • VantaDB"
  },
  {
    id: "03",
    title: "IDE Assistants & Tooling",
    industry: "DEVELOPER TOOLS",
    desc: "Embed cognitive memory into desktop applications and IDE plugins. Blazing fast code-search running entirely in-process.",
    stack: "Electron/Tauri • Rust • VantaDB"
  }
];

export function SwissUseCases() {
  const sectionRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
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
          }
        }
      );
    },
    { scope: sectionRef }
  );

  return (
    <section 
      ref={sectionRef} 
      className="swiss-section" 
      style={{ background: "var(--background)", paddingTop: "80px", paddingBottom: "160px" }}
    >
      <div className="swiss-inner">
        <div style={{ marginBottom: "64px" }}>
          <h2 style={{ fontFamily: "var(--font-display)", fontSize: "var(--text-display)", fontWeight: 700, margin: "0", letterSpacing: "-0.04em", color: "var(--foreground)" }}>
            Applied Use Cases.
          </h2>
        </div>

        <div style={{ display: "flex", flexDirection: "column", borderTop: "1px solid var(--border)" }}>
          {CASES.map((uc) => (
            <div 
              key={uc.id}
              className="swiss-uc-card"
              style={{
                display: "grid",
                gridTemplateColumns: "3fr 9fr",
                padding: "48px 0",
                borderBottom: "1px solid var(--border)",
                transition: "all 200ms",
                cursor: "default"
              }}
              onMouseEnter={(e) => {
                const num = e.currentTarget.querySelector('.swiss-uc-num');
                if (num) { (num as HTMLElement).style.color = "var(--amber)"; }
              }}
              onMouseLeave={(e) => {
                const num = e.currentTarget.querySelector('.swiss-uc-num');
                if (num) { (num as HTMLElement).style.color = "var(--subtle, #e0e0e0)"; }
              }}
            >
              {/* Columna Izquierda: Índice numérico grande */}
              <div>
                <span 
                  className="swiss-uc-num"
                  style={{ 
                    fontFamily: "var(--font-display)", 
                    fontSize: "var(--text-display)", 
                    fontWeight: 700, 
                    color: "var(--subtle, #e0e0e0)",
                    transition: "color 150ms"
                  }}
                >
                  {uc.id}
                </span>
              </div>
              
              {/* Columna Derecha: Contenido */}
              <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.75rem", color: "var(--steel)", letterSpacing: "0.1em" }}>
                  [{uc.industry}]
                </span>
                
                <h3 style={{ margin: 0, fontFamily: "var(--font-display)", fontSize: "2rem", fontWeight: 600, color: "var(--foreground)", letterSpacing: "-0.02em" }}>
                  {uc.title}
                </h3>
                
                <p style={{ margin: 0, fontFamily: "var(--font-sans)", fontSize: "1.1rem", color: "var(--muted)", lineHeight: 1.6, maxWidth: "80%" }}>
                  {uc.desc}
                </p>

                <div style={{ marginTop: "16px" }}>
                  <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.85rem", color: "var(--foreground)", background: "var(--surface-raised, #f0f0f0)", padding: "4px 8px" }}>
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

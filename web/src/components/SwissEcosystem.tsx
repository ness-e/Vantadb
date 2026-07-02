import { useRef } from "react";
import { gsap, useGSAP, ScrollTrigger } from "../lib/gsap";

const INTEGRATIONS = [
  { id: "pydantic", name: "Pydantic AI", category: "FRAMEWORK" },
  { id: "langchain", name: "LangChain", category: "FRAMEWORK" },
  { id: "llamaindex", name: "LlamaIndex", category: "FRAMEWORK" },
  { id: "crewai", name: "CrewAI", category: "AGENTS" },
  { id: "autogen", name: "AutoGen", category: "AGENTS" },
  { id: "smolagents", name: "smolagents", category: "AGENTS" },
  { id: "openai", name: "OpenAI", category: "MODELS" },
  { id: "anthropic", name: "Anthropic", category: "MODELS" },
  { id: "fastapi", name: "FastAPI", category: "DEPLOY" },
  { id: "modal", name: "Modal", category: "DEPLOY" },
  { id: "ray", name: "Ray Serve", category: "DEPLOY" },
  { id: "streamlit", name: "Streamlit", category: "UI" },
];

export function SwissEcosystem() {
  const sectionRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      gsap.fromTo(
        ".swiss-eco-cell",
        { opacity: 0, scale: 0.95 },
        {
          opacity: 1,
          scale: 1,
          duration: 0.4,
          stagger: 0.05,
          ease: "power2.out",
          scrollTrigger: {
            trigger: ".swiss-eco-grid",
            start: "top 80%",
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
      style={{ background: "var(--background)", paddingTop: "120px", paddingBottom: "160px" }}
    >
      <div className="swiss-inner">
        <div style={{ marginBottom: "64px" }}>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-label)", fontWeight: 600, letterSpacing: "0.14em", color: "var(--steel)", textTransform: "uppercase" }}>
            [ECOSYSTEM]
          </span>
          <h2 style={{ fontFamily: "var(--font-display)", fontSize: "var(--text-display)", fontWeight: 700, margin: "24px 0 0 0", letterSpacing: "-0.04em", color: "var(--foreground)" }}>
            Integration Matrix.
          </h2>
        </div>

        <div className="swiss-eco-grid" style={{ 
          display: "grid", 
          gridTemplateColumns: "repeat(auto-fit, minmax(250px, 1fr))", 
          gap: "1px", 
          background: "var(--border)", 
          border: "1px solid var(--border)" 
        }}>
          {INTEGRATIONS.map((int) => (
            <div 
              key={int.id}
              className="swiss-eco-cell"
              style={{
                background: "var(--surface)",
                padding: "32px 24px",
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                gap: "16px",
                transition: "all 150ms",
                cursor: "pointer"
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = "var(--amber-dim, rgba(255,85,0,0.05))";
                const svg = e.currentTarget.querySelector('svg');
                if (svg) { svg.style.stroke = "var(--amber)"; }
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = "var(--surface)";
                const svg = e.currentTarget.querySelector('svg');
                if (svg) { svg.style.stroke = "var(--steel)"; }
              }}
            >
              {/* Icono generico monoline de integración */}
              <svg 
                width="32" height="32" viewBox="0 0 24 24" 
                fill="none" stroke="var(--steel)" strokeWidth="1.5"
                style={{ transition: "stroke 150ms" }}
              >
                <rect x="4" y="4" width="16" height="16" />
                <path d="M4 12h16M12 4v16" />
                <circle cx="12" cy="12" r="3" fill="var(--background)" />
              </svg>

              <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.9rem", fontWeight: 600, color: "var(--foreground)" }}>
                {int.name}
              </span>
              
              <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.7rem", color: "var(--muted)", letterSpacing: "0.1em" }}>
                [{int.category}]
              </span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

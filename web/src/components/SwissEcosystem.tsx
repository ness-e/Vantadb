import { useRef } from "react";
import { gsap, useGSAP } from "../lib/gsap";

const CATEGORIES = [
  {
    label: "FRAMEWORK",
    items: ["Pydantic AI", "LangChain", "LlamaIndex"],
  },
  {
    label: "AGENTS",
    items: ["CrewAI", "AutoGen", "smolagents"],
  },
  {
    label: "MODELS",
    items: ["OpenAI", "Anthropic", "Ollama"],
  },
  {
    label: "DEPLOY",
    items: ["FastAPI", "Modal", "Ray Serve"],
  },
  {
    label: "UI",
    items: ["Streamlit", "Gradio"],
  },
];

export function SwissEcosystem() {
  const sectionRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      gsap.fromTo(
        ".swiss-eco-row",
        { opacity: 0, y: 12 },
        {
          opacity: 1,
          y: 0,
          duration: 0.5,
          stagger: 0.06,
          ease: "cubic-bezier(0.25, 1, 0.5, 1)",
          scrollTrigger: {
            trigger: sectionRef.current,
            start: "top 75%",
          },
        }
      );
    },
    { scope: sectionRef }
  );

  return (
    <section
      ref={sectionRef}
      className="swiss-section"
      style={{
        background: "var(--background)",
        paddingTop: "120px",
        paddingBottom: "160px",
      }}
    >
      <div className="swiss-inner">
        {/* Eyebrow — 2/3 budget */}
        <span
          style={{
            fontFamily: "var(--font-sans)",
            fontSize: "0.95rem",
            fontStyle: "italic",
            fontWeight: 500,
            letterSpacing: "0.02em",
            color: "var(--steel)",
          }}
        >
          Ecosystem Matrix
        </span>
        <h2
          style={{
            fontFamily: "var(--font-display)",
            fontSize: "var(--text-display)",
            fontWeight: 700,
            margin: "16px 0 80px 0",
            letterSpacing: "-0.04em",
            color: "var(--foreground)",
          }}
        >
          Integration Matrix.
        </h2>

        {/* Category rows — grouped layout, NOT grid */}
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            borderTop: "1px solid var(--border)",
          }}
        >
          {CATEGORIES.map((cat) => (
            <div
              key={cat.label}
              className="swiss-eco-row"
              style={{
                display: "grid",
                gridTemplateColumns: "160px 1fr",
                padding: "32px 0",
                borderBottom: "1px solid var(--border)",
                alignItems: "center",
                gap: "24px",
              }}
            >
              {/* Category label */}
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "var(--text-label)",
                  fontWeight: 600,
                  color: "var(--steel)",
                  letterSpacing: "0.1em",
                  textTransform: "uppercase" as const,
                }}
              >
                [{cat.label}]
              </span>

              {/* Integration chips */}
              <div
                style={{
                  display: "flex",
                  flexWrap: "wrap",
                  gap: "8px",
                }}
              >
                {cat.items.map((item) => (
                  <span
                    key={item}
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.88rem",
                      fontWeight: 500,
                      color: "var(--foreground)",
                      padding: "8px 16px",
                      border: "1px solid var(--border)",
                      background: "var(--surface)",
                      transition:
                        "all 150ms cubic-bezier(0.25, 1, 0.5, 1)",
                      cursor: "default",
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.borderColor = "var(--amber)";
                      e.currentTarget.style.color = "var(--amber)";
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.borderColor = "var(--border)";
                      e.currentTarget.style.color = "var(--foreground)";
                    }}
                  >
                    {item}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

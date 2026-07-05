import { useRef } from "react";
import { gsap, useGSAP } from "../lib/gsap";
import "../styles/ecosystem.css";

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

export function NbEcosystem() {
  const sectionRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const cells = gsap.utils.toArray<HTMLElement>(".eco-cell");
        if (!cells.length) return;

        gsap.fromTo(
          cells,
          { opacity: 0, y: 20, transformOrigin: "top center" },
          {
            opacity: 1,
            y: 0,
            duration: 0.35,
            stagger: 0.05,
            ease: "cubic-bezier(0.05, 0.95, 0.3, 1)",
            scrollTrigger: {
              trigger: sectionRef.current,
              start: "top 75%",
            },
          },
        );
      });
    },
    { scope: sectionRef },
  );

  return (
    <section ref={sectionRef} className="nb-section" aria-label="Integration ecosystem">
      <div className="nb-inner">
        <div className="nb-frame" data-frame-label="INTEGRATION MATRIX">
          <div className="nb-section-header">
            <h2 className="eco-heading">Integration Matrix.</h2>
          </div>

          <div className="eco-grid" role="list">
            {CATEGORIES.map((cat) => (
              <div key={cat.label} className="eco-cell" role="listitem">
                <span className="eco-category-label">{cat.label}</span>
                <div className="eco-items">
                  {cat.items.map((item) => (
                    <span key={item} className="eco-item">
                      {item}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

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
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
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
          },
        );
      });
    },
    { scope: sectionRef },
  );

  return (
    <section
      ref={sectionRef}
      className="swiss-section eco-section"
    >
      <div className="swiss-inner">
        {/* Eyebrow — 2/3 budget */}
        <span className="eco-eyebrow">
          Ecosystem Matrix
        </span>
        <h2 className="eco-heading">
          Integration Matrix.
        </h2>

        {/* Category rows — grouped layout, NOT grid */}
        <div className="eco-list">
          {CATEGORIES.map((cat) => (
            <div
              key={cat.label}
              className="swiss-eco-row"
            >
              {/* Category label */}
              <span className="eco-label">
                [{cat.label}]
              </span>

              {/* Integration chips */}
              <div className="eco-chips">
                {cat.items.map((item) => (
                  <span
                    key={item}
                    className="eco-chip"
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

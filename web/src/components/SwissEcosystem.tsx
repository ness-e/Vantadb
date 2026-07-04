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
          { scaleX: 0, opacity: 0, transformOrigin: "left center" },
          {
            scaleX: 1,
            opacity: 1,
            duration: 0.35,
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
      aria-label="Integration ecosystem"
    >
      <div className="swiss-inner">
        <h2 className="eco-heading">Integration Matrix.</h2>

        <div className="eco-list" role="list">
          {CATEGORIES.map((cat) => (
            <div key={cat.label} className="swiss-eco-row" role="listitem">
              <span className="eco-label">[{cat.label}]</span>

              <div className="eco-chips">
                {cat.items.map((item) => (
                  <span key={item} className="eco-chip">
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

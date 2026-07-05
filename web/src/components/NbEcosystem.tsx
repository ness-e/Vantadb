import { useRef } from "react";
import { gsap, useGSAP } from "../lib/gsap";
import "../styles/ecosystem.css";

const CATEGORIES = [
  { label: "FRAMEWORK", items: ["Pydantic AI", "LangChain", "LlamaIndex"] },
  { label: "AGENTS", items: ["CrewAI", "AutoGen", "smolagents"] },
  { label: "MODELS", items: ["OpenAI", "Anthropic", "Ollama"] },
  { label: "DEPLOY", items: ["FastAPI", "Modal", "Ray Serve"] },
  { label: "UI", items: ["Streamlit", "Gradio"] },
];

export function NbEcosystem() {
  const sectionRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const cols = gsap.utils.toArray<HTMLElement>(".nb-eco-col");
        if (!cols.length) return;

        gsap.fromTo(
          cols,
          { opacity: 0, y: 20 },
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
    <section ref={sectionRef} className="nb-section" aria-label="Ecosystem">
      <div className="nb-inner">
        <span className="nb-mono-label">[INTEGRATIONS]</span>
        <h2 className="nb-section-headline">Works with your stack.</h2>

        <div className="nb-eco-matrix">
          {CATEGORIES.map((cat) => (
            <div key={cat.label} className="nb-eco-col">
              <span className="nb-eco-col-label">{cat.label}</span>
              <div className="nb-eco-items">
                {cat.items.map((item) => (
                  <span key={item} className="nb-eco-item">{item}</span>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

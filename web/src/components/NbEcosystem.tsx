import { useRef } from "react";
import { gsap } from "../lib/gsap";
import { useAnimationSafe } from "../hooks/useAnimationSafe";
import { scrollTriggerConfig } from "../lib/gsap-utils";
import { NbSection, NbSectionHeader } from "../components/nb";
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

  useAnimationSafe(() => {
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
        scrollTrigger: scrollTriggerConfig(sectionRef.current),
      },
    );
  }, sectionRef);

  return (
    <NbSection ref={sectionRef} ariaLabel="Ecosystem">
      <NbSectionHeader monoLabel="[INTEGRATIONS]" headline="Works with your stack." />
      <div className="nb-eco-matrix">
        {CATEGORIES.map((cat) => (
          <div key={cat.label} className="nb-eco-col">
            <span className="nb-eco-col-label">{cat.label}</span>
            <div className="nb-eco-items">
              {cat.items.map((item) => (
                <span key={item} className="nb-eco-tag">
                  {item}
                </span>
              ))}
            </div>
          </div>
        ))}
      </div>
    </NbSection>
  );
}

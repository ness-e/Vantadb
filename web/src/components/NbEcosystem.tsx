import { NbSection, NbSectionHeader } from "../components/nb";
import "../styles/ecosystem.css";

const CATEGORIES = [
  {
    label: "FRAMEWORK", color: "var(--amber)",
    items: [
      { name: "Pydantic AI", port: "A1" },
      { name: "LangChain", port: "A2" },
      { name: "LlamaIndex", port: "A3" },
    ],
  },
  {
    label: "AGENTS", color: "var(--success)",
    items: [
      { name: "CrewAI", port: "B1" },
      { name: "AutoGen", port: "B2" },
      { name: "smolagents", port: "B3" },
    ],
  },
  {
    label: "MODELS", color: "var(--amber)",
    items: [
      { name: "OpenAI", port: "C1" },
      { name: "Anthropic", port: "C2" },
      { name: "Ollama", port: "C3" },
    ],
  },
  {
    label: "DEPLOY", color: "var(--steel)",
    items: [
      { name: "FastAPI", port: "D1" },
      { name: "Modal", port: "D2" },
      { name: "Ray Serve", port: "D3" },
    ],
  },
  {
    label: "UI", color: "var(--muted)",
    items: [
      { name: "Streamlit", port: "E1" },
      { name: "Gradio", port: "E2" },
    ],
  },
];

export function NbEcosystem() {
  return (
    <NbSection ariaLabel="Ecosystem">
      <NbSectionHeader
        monoLabel="[PATCH BAY]"
        headline="Plug into your stack."
        sub="Every port is live. Connect VantaDB to your existing toolchain."
      />

      <div className="nb-patch-matrix">
        {CATEGORIES.map((cat) => (
          <div key={cat.label} className="nb-patch-col">
            <div className="nb-patch-col-label" style={{ borderBottomColor: cat.color }}>
              <span className="nb-patch-col-dot" style={{ background: cat.color }} />
              {cat.label}
            </div>
            <div className="nb-patch-items">
              {cat.items.map((item) => (
                <div key={item.name} className="nb-patch-item">
                  <div className="nb-patch-port" style={{ borderColor: cat.color }}>
                    <span className="nb-patch-port-inner" style={{ background: cat.color }} />
                  </div>
                  <div className="nb-patch-item-info">
                    <span className="nb-patch-item-name">{item.name}</span>
                    <span className="nb-patch-item-port">{item.port}</span>
                  </div>
                  <svg width="8" height="8" viewBox="0 0 8 8" fill="none" aria-hidden="true" className="nb-patch-plug">
                    <rect x="1" y="1" width="6" height="6" stroke={cat.color} strokeWidth="1" fill="none" />
                    <line x1="4" y1="0" x2="4" y2="8" stroke={cat.color} strokeWidth="1" />
                  </svg>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </NbSection>
  );
}

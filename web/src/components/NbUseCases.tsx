import { NbSection, NbSectionHeader } from "../components/nb";
import "../styles/use-cases.css";

const CASES = [
  {
    id: "01", title: "AI Agent Memory",
    desc: "Persistent context windows for distributed agent swarms without network overhead. Local execution means zero API latency.",
    tags: ["AGENTS", "MEMORY", "LOCAL"],
    signal: "LIVE",
  },
  {
    id: "02", title: "Local RAG Pipeline",
    desc: "Complete semantic search on secure enterprise hardware. Air-gapped environments fully supported.",
    tags: ["RAG", "ENTERPRISE", "AIR-GAP"],
    signal: "LIVE",
  },
  {
    id: "03", title: "IDE Code Intelligence",
    desc: "Embed cognitive memory into desktop applications and IDE plugins. Blazing fast code-search in-process.",
    tags: ["IDE", "PLUGIN", "DESKTOP"],
    signal: "BETA",
  },
  {
    id: "04", title: "Offline Knowledge Base",
    desc: "Edge-deployed semantic search without internet. Perfect for field devices, kiosks, disconnected environments.",
    tags: ["EDGE", "OFFLINE", "KIOSK"],
    signal: "PREVIEW",
  },
];

export function NbUseCases() {
  return (
    <NbSection variant="lg" ariaLabel="Use cases">
      <NbSectionHeader
        monoLabel="[SIGNALS]"
        headline="Agent memory, local RAG, IDE intelligence."
        sub="Four primary deployment patterns — each with a distinct signal status."
      />

      <div className="nb-sc-grid">
        {CASES.map((uc, i) => (
          <article key={uc.id} className={`nb-sc-card${i === 0 ? " nb-sc-card--featured" : ""}`}>
            {/* Signal indicator header */}
            <div className="nb-sc-header">
              <span className="nb-sc-num">{uc.id}</span>
              <div className="nb-sc-line" />
              <span className={`nb-sc-signal nb-sc-signal--${uc.signal.toLowerCase()}`}>
                <span className="nb-sc-signal-dot" />
                {uc.signal}
              </span>
            </div>

            <div className="nb-sc-body">
              <h3 className="nb-sc-title">{uc.title}</h3>
              <p className="nb-sc-desc">{uc.desc}</p>
            </div>

            {/* Tags footer */}
            <div className="nb-sc-tags">
              {uc.tags.map((t) => (
                <span key={t} className="nb-sc-tag">{t}</span>
              ))}
            </div>

            <span className="nb-arrow nb-sc-arrow">Learn more</span>
          </article>
        ))}
      </div>
    </NbSection>
  );
}

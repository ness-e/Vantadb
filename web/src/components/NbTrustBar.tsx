import { memo } from "react";
import "../styles/trust-bar.css";

const INTEGRATIONS = ["RUST", "PYTHON", "DOCKER", "GITHUB", "VSCODE"];

export const NbTrustBar = memo(function NbTrustBar() {
  return (
    <section className="nb-section nb-section--sm" aria-label="Trust indicators">
      <div className="nb-inner">
        <span
          className="nb-mono-label"
          style={{ display: "block", textAlign: "center", marginBottom: "var(--space-md)" }}
        >
          TRUSTED BY ENGINEERS AT
        </span>
        <div className="nb-trust-grid">
          {INTEGRATIONS.map((name) => (
            <div key={name} className="nb-trust-item">
              {name}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
});

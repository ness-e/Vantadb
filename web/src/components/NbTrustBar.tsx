import { memo } from "react";
import "../styles/trust-bar.css";
import { NbSection } from "./nb";

const INTEGRATIONS = ["RUST", "PYTHON", "DOCKER", "GITHUB", "VSCODE"];

export const NbTrustBar = memo(function NbTrustBar() {
  return (
    <NbSection variant="sm" ariaLabel="Trust indicators">
      <span className="nb-mono-label nb-trust-label">TRUSTED BY ENGINEERS AT</span>
      <div className="nb-trust-grid">
        {INTEGRATIONS.map((name) => (
          <div key={name} className="nb-trust-item">
            {name}
          </div>
        ))}
      </div>
    </NbSection>
  );
});

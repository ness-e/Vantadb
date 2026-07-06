import { memo } from "react";
import "../styles/trust-bar.css";
import { NbSection } from "./nb";

const TICKER_DATA = [
  { value: "2,847", label: "GITHUB STARS" },
  { value: "18K+", label: "PYPI DOWNLOADS" },
  { value: "4.2K+", label: "CRATES.IO" },
  { value: "v0.1.5", label: "LATEST" },
  { value: "844", label: "COMMITS" },
  { value: "12", label: "CONTRIBUTORS" },
  { value: "RUST", label: "CORE" },
  { value: "PYTHON", label: "SDK" },
  { value: "MIT", label: "LICENSE" },
  { value: "2MB", label: "BINARY" },
  { value: "0", label: "SERVERS" },
];

export const NbTrustBar = memo(function NbTrustBar() {
  return (
    <NbSection variant="sm" ariaLabel="Trust indicators">
      <div className="nb-ticker-wrap">
        <div className="nb-ticker-track" aria-hidden="true">
          {[...TICKER_DATA, ...TICKER_DATA, ...TICKER_DATA].map((item, i) => (
            <div key={i} className="nb-ticker-item">
              <span className="nb-ticker-item-value">{item.value}</span>
              <span className="nb-ticker-item-label">{item.label}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="nb-ticker-integrations">
        <span className="nb-mono-label nb-ticker-intro">TRUSTED BY ENGINEERS</span>
        <div className="nb-ticker-labels">
          {["RUST", "PYTHON", "DOCKER", "GITHUB", "VSCODE", "LANGCHAIN", "LLAMAINDEX"].map(
            (name) => (
              <span key={name} className="nb-ticker-label">
                {name}
              </span>
            ),
          )}
        </div>
      </div>
    </NbSection>
  );
});

import { memo } from "react";
import "../styles/trust-bar.css";

const INTEGRATIONS = ["RUST", "PYTHON", "DOCKER", "GITHUB", "VSCODE"];

const TICKER_ITEMS = [
  "INSERT 10k vectors... DONE (1.2ms)",
  "QUERY 1k results... DONE (0.8ms)",
  "HYBRID SEARCH 0.998 R@10",
  "BUILD TIME 47ms",
];

export const NbTrustBar = memo(function NbTrustBar() {
  return (
    <section className="nb-section nb-section--sm" aria-label="Trust indicators">
      <div className="nb-inner">
        <div className="nb-trust-header">
          <span className="nb-trust-stars">2,847 ★</span>
          <span className="nb-trust-label">USED BY</span>
        </div>

        <div className="nb-trust-integrations">
          {INTEGRATIONS.map((name) => (
            <div key={name} className="nb-trust-box" aria-hidden="true">
              {name}
            </div>
          ))}
        </div>

        <div className="nb-trust-marquee nb-trust-marquee-container">
          <div className="nb-trust-marquee-track">
            <span>{TICKER_ITEMS.join(" /// ")}</span>
            <span>{TICKER_ITEMS.join(" /// ")}</span>
          </div>
        </div>
      </div>
    </section>
  );
});

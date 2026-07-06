import { memo } from "react";
import "../styles/trust-bar.css";

const METRICS = [
  { value: "1.2", unit: "ms", label: "P50 QUERY LATENCY" },
  { value: "2", unit: "MB", label: "BINARY FOOTPRINT" },
  { value: "0", unit: "", label: "SERVERS REQUIRED" },
  { value: "99.8", unit: "%", label: "RECALL@10 (HNSW)" },
];

const INTEGRATIONS = ["RUST", "PYTHON", "DOCKER", "GITHUB", "VSCODE"];

export const NbDataTrust = memo(function NbDataTrust() {
  return (
    <section className="nb-data-trust" aria-label="Data and trust indicators">
      <div className="nb-inner">
        <div className="nb-data-trust-layout">
          <h2 className="nb-data-trust-headline">
            VantaDB<br />
            <span className="nb-data-trust-headline-accent">by the numbers</span>
          </h2>
          <div className="nb-data-trust-grid">
            {METRICS.map((m) => (
              <div key={m.label} className="nb-data-trust-card">
                <span className="nb-data-trust-val">
                  {m.value}
                  {m.unit && <span className="nb-data-trust-unit">{m.unit}</span>}
                </span>
                <span className="nb-data-trust-label">{m.label}</span>
              </div>
            ))}
          </div>
        </div>
        <div className="nb-data-trust-brand-strip">
          {INTEGRATIONS.map((name) => (
            <span key={name} className="nb-data-trust-brand">{name}</span>
          ))}
        </div>
        <p className="nb-data-trust-footnote">* Core Rust benchmarks. Python SDK performance may vary.</p>
      </div>
    </section>
  );
});

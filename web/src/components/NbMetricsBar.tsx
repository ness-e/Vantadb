import { memo } from "react";
import "../styles/metrics-bar.css";
import { NbSplitFlap } from "./nb/NbSplitFlap";

const METRICS = [
  { value: "1.2", suffix: "ms", label: "p50 QUERY LATENCY" },
  { value: "2", suffix: "MB", label: "BINARY FOOTPRINT" },
  { value: "0", suffix: "", label: "SERVERS REQUIRED" },
  { value: "99.8", suffix: "%", label: "RECALL@10 (HNSW)" },
];

export const NbMetricsBar = memo(function NbMetricsBar() {
  return (
    <section className="nb-section" aria-label="Key metrics">
      <div className="nb-inner">
        <div className="nb-asymmetric">
          <div>
            <span className="nb-mono-label">[PERFORMANCE DATA]</span>
            <h2 className="nb-section-headline">Built for speed.</h2>
            <p className="nb-section-sub">
              Every number speaks for itself — no fluff, no marketing.
            </p>
          </div>
          <div className="nb-metrics-grid">
            {METRICS.map((m) => (
              <div key={m.label} className="nb-metric-cell">
                <span className="nb-metric-value">
                  <NbSplitFlap value={m.value} />
                  {m.suffix}
                </span>
                <span className="nb-metric-desc">{m.label}</span>
              </div>
            ))}
          </div>
        </div>
        <p className="nb-metrics-footnote">
          * Core Rust benchmarks. Python SDK performance may vary.
        </p>
      </div>
    </section>
  );
});

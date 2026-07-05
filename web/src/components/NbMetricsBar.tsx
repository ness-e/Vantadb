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
    <section className="nb-section nb-section--sm" aria-label="Key metrics">
      <div className="nb-inner">
        <div>
          <div className="nb-metrics-strip">
            {METRICS.map((m) => (
              <div key={m.label} className="nb-metrics-cell">
                <span className="nb-metrics-value nb-metrics-value--tabular">
                  <NbSplitFlap value={m.value} />
                  {m.suffix}
                </span>
                <span className="nb-metrics-label">{m.label}</span>
              </div>
            ))}
          </div>
          <p className="nb-metrics-disclaimer">
            * Core Rust benchmarks. Python SDK performance may vary.
          </p>
        </div>
      </div>
    </section>
  );
});

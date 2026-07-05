import { memo } from "react";
import "../styles/metrics-bar.css";
import { NbSplitFlap } from "./nb/NbSplitFlap";
import { NbSection, NbSectionHeader } from "./nb";

const METRICS = [
  { value: "1.2", suffix: "ms", label: "p50 QUERY LATENCY", featured: true },
  { value: "2", suffix: "MB", label: "BINARY FOOTPRINT", featured: false },
  { value: "0", suffix: "", label: "SERVERS REQUIRED", featured: false },
  { value: "99.8", suffix: "%", label: "RECALL@10 (HNSW)", featured: false },
];

export const NbMetricsBar = memo(function NbMetricsBar() {
  return (
    <NbSection ariaLabel="Key metrics">
      <div className="nb-metrics-lead">
        <NbSectionHeader
          monoLabel="[PERFORMANCE DATA]"
          headline="Built for speed."
          sub="Every number speaks for itself — no fluff, no marketing."
        />
        <div className="nb-metrics-grid">
          {METRICS.map((m) => (
            <div
              key={m.label}
              className={`nb-card-frame${m.featured ? " nb-card-frame--featured" : ""}`}
            >
              <div className="nb-metric-val">
                <NbSplitFlap value={m.value} />
                <span className="nb-metric-unit">{m.suffix}</span>
              </div>
              <span className="nb-metric-label">{m.label}</span>
            </div>
          ))}
        </div>
      </div>
      <p className="nb-metrics-footnote">
        * Core Rust benchmarks. Python SDK performance may vary.
      </p>
    </NbSection>
  );
});

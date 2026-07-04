import { memo } from "react";

const METRICS = [
  { value: "1.2ms", label: "p50 QUERY LATENCY" },
  { value: "2MB", label: "BINARY FOOTPRINT" },
  { value: "0", label: "SERVERS REQUIRED" },
  { value: "99.8%", label: "RECALL@10 (HNSW)" },
];

export const SwissMetricsBar = memo(function SwissMetricsBar() {
  return (
    <section className="swiss-metrics-bar" aria-label="Key metrics">
      <div className="swiss-inner">
        <div className="swiss-metrics-bar-strip">
          {METRICS.map((m, i) => (
            <div key={m.label} className="swiss-metrics-bar-item">
              <span className="swiss-metrics-bar-value">{m.value}</span>
              <span className="swiss-metrics-bar-label">{m.label}</span>
              {i < METRICS.length - 1 && (
                <div className="swiss-metrics-bar-sep" aria-hidden="true" />
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
});

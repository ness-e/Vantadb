import { memo } from "react";
import "../styles/metrics-bar.css";

const METRICS = [
  { value: "1.2ms", label: "p50 QUERY LATENCY" },
  { value: "2MB", label: "BINARY FOOTPRINT" },
  { value: "0", label: "SERVERS REQUIRED" },
  { value: "99.8%", label: "RECALL@10 (HNSW)" },
];

export const NbMetricsBar = memo(function NbMetricsBar() {
  return (
    <section className="nb-section nb-section--sm" aria-label="Key metrics">
      <div className="nb-inner">
        <div className="nb-frame" data-frame-label="METRICS">
          <div className="nb-metrics-strip">
            {METRICS.map((m) => (
              <div key={m.label} className="nb-metrics-cell">
                <span className="nb-metrics-value">&gt; {m.value}</span>
                <span className="nb-metrics-label">{m.label}</span>
              </div>
            ))}
          </div>
          <div className="nb-telemetry" aria-hidden="true">
            <span>last_updated: live</span>
            <span>source: telemetry/vantadb</span>
            <span>status: operational</span>
          </div>
        </div>
      </div>
    </section>
  );
});

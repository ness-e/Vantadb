import { memo } from "react";
import "../styles/metrics-bar.css";
import { NbSection, NbSectionHeader } from "./nb";

const CERTIFIED = [
  {
    value: "1.2",
    unit: "ms",
    label: "p50 Query Latency",
    note: "HNSW, 10K×128d, Rust Core",
    bar: 100,
  },
  {
    value: "0.998",
    unit: "",
    label: "Recall@10",
    note: "SIFT1M, 100K×128d, Cosine",
    bar: 99,
  },
  {
    value: "1,172",
    unit: "B/vec",
    label: "Memory Efficiency",
    note: "HNSW + BM25 + Metadata",
    bar: 78,
  },
];

const COMPETITION = [
  { engine: "VantaDB", insert: 8450, bar: 100 },
  { engine: "Qdrant", insert: 4100, bar: 49 },
  { engine: "ChromaDB", insert: 1200, bar: 14 },
];

const SPEEDUPS = [
  { value: "2.8x", label: "Index Build (L2)" },
  { value: "4.01x", label: "Batch Search" },
  { value: "100%", label: "Crash Survival" },
];

export const NbMetricsBar = memo(function NbMetricsBar() {
  return (
    <NbSection ariaLabel="Key metrics">
      <NbSectionHeader
        monoLabel="[PERFORMANCE DATA]"
        headline="2MB binary. 1.2ms p50. Zero servers."
        sub="Every number speaks for itself — no fluff, no marketing."
      />

      <div className="nb-perf-doc">
        <div className="nb-perf-doc-header">
          <span className="nb-perf-doc-id">CERT-2026-001</span>
          <span className="nb-perf-doc-title">
            VantaDB Core Engine — Certified Benchmark Report
          </span>
          <span className="nb-perf-doc-seal">CERTIFIED</span>
        </div>

        {/* ── Certified stats ── */}
        <div className="nb-perf-certified">
          {CERTIFIED.map((c) => (
            <div key={c.label} className="nb-perf-cert-row">
              <div className="nb-perf-cert-metric">
                <span className="nb-perf-cert-value">
                  {c.value}
                  {c.unit && <span className="nb-perf-cert-unit">{c.unit}</span>}
                </span>
                <div className="nb-perf-cert-info">
                  <span className="nb-perf-cert-label">{c.label}</span>
                  <span className="nb-perf-cert-note">{c.note}</span>
                </div>
              </div>
              <div className="nb-perf-cert-bar-track">
                <div className="nb-perf-cert-bar-fill" style={{ width: `${c.bar}%` }} />
              </div>
            </div>
          ))}
        </div>

        {/* ── vs Competition ── */}
        <div className="nb-perf-vs">
          <span className="nb-perf-vs-label">vs Competition — Insert Throughput (QPS)</span>
          {COMPETITION.map((c) => {
            const isVanta = c.engine === "VantaDB";
            return (
              <div
                key={c.engine}
                className={`nb-perf-vs-row${isVanta ? " nb-perf-vs-row--win" : ""}`}
              >
                <span className="nb-perf-vs-name">{c.engine}</span>
                <div className="nb-perf-vs-bar-track">
                  <div className="nb-perf-vs-bar-fill" style={{ width: `${c.bar}%` }} />
                </div>
                <span className="nb-perf-vs-num">{c.insert.toLocaleString()}</span>
              </div>
            );
          })}
        </div>

        {/* ── Speedups row ── */}
        <div className="nb-perf-speedups">
          {SPEEDUPS.map((s) => (
            <div key={s.label} className="nb-perf-speedup-badge">
              <span className="nb-perf-speedup-value">{s.value}</span>
              <span className="nb-perf-speedup-label">{s.label}</span>
            </div>
          ))}
        </div>

        {/* ── Footer ── */}
        <p className="nb-perf-footnote">
          * Core Rust benchmarks on AMD Ryzen 12-Core @ 3.5GHz, AVX2. SIFT1M dataset. Python SDK
          performance may vary due to PyO3 FFI overhead.
        </p>
      </div>
    </NbSection>
  );
});

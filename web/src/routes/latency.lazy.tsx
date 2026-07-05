import { createLazyRoute } from "@tanstack/react-router";
import { useState, useMemo } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/latency")({
  component: LatencyPage,
  pendingComponent: PendingComponent,
});

const BREAKDOWN = [
  { label: "Network", legacy: "70ms", vanta: "0ms", legacyW: 35, vantaW: 0 },
  { label: "Index search", legacy: "50ms", vanta: "0.6ms", legacyW: 25, vantaW: 50 },
  { label: "Serialize", legacy: "25ms", vanta: "0ms", legacyW: 15, vantaW: 0 },
  { label: "Cache fill", legacy: "45ms", vanta: "0.6ms", legacyW: 20, vantaW: 50 },
];

const LEGACY_ITEMS = [
  "Network round-trip: 50–80ms (TLS + serialization)",
  "Remote index traversal: 40–60ms",
  "Result serialization: 20–30ms",
  "Cache miss penalty: 100ms+ to S3 fallback",
  "Cold start: 2–5s (serverless DB wake)",
];

const VANTA_ITEMS = [
  "Zero network: same-process memory access",
  "HNSW graph traversal: 0.4–0.8ms",
  "BM25 intersection: 0.2–0.4ms",
  "No serialization: zero-copy result passing",
  "No cold start: process is always warm",
];

function LatencyPage() {
  const [pipelineSize, setPipelineSize] = useState(50);
  const [mode, setMode] = useState<"rust" | "python">("rust");
  const vantaLatency = useMemo(() => (mode === "rust" ? 1.2 : 39.74), [mode]);
  const legacyTotal = useMemo(() => pipelineSize * 200, [pipelineSize]);
  const vantaTotal = useMemo(() => pipelineSize * vantaLatency, [pipelineSize, vantaLatency]);
  const speedup = useMemo(
    () => Math.round(legacyTotal / Math.max(vantaTotal, 0.1)),
    [legacyTotal, vantaTotal],
  );

  return (
    <div>
      <NbSubpageHero
        num="07"
        eyebrow="Latency Performance"
        title={
          <span>
            1.2ms p50 (Rust Core).
            <br />
            No network.
          </span>
        }
        sub="VantaDB runs in your process — no network round-trip, no serialization overhead, no cold starts. Every microsecond matters when your agent is waiting. Rust Core: 1.2ms p50 / Python SDK: ~39.74ms p50"
      />

      <div className="flex justify-center gap-2 mt-6 mb-0">
        <button
          onClick={() => setMode("rust")}
          className="font-mono text-[0.72rem] px-5 py-2 cursor-pointer tracking-[0.04em]"
          style={{
            border: mode === "rust" ? "2px solid var(--amber)" : "1px solid var(--border)",
            background: mode === "rust" ? "var(--surface-alt)" : "transparent",
            color: mode === "rust" ? "var(--amber)" : "var(--muted)",
          }}
        >
          Rust Core — 1.2ms p50
        </button>
        <button
          onClick={() => setMode("python")}
          className="font-mono text-[0.72rem] px-5 py-2 cursor-pointer tracking-[0.04em]"
          style={{
            border: mode === "python" ? "2px solid var(--amber)" : "1px solid var(--border)",
            background: mode === "python" ? "var(--surface-alt)" : "transparent",
            color: mode === "python" ? "var(--amber)" : "var(--muted)",
          }}
        >
          Python SDK — ~39.74ms p50
        </button>
      </div>

      <main>
        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-label">01 / 03 — Comparison</div>

            <div className="nb-grid nb-grid--cols-2" style={{ marginTop: "3rem" }}>
              <div className="nb-cell">
                <div className="nb-label" style={{ color: "var(--steel)" }}>
                  LEGACY — ~200ms
                </div>
                <ul
                  className="flex flex-col gap-3 mt-4"
                  style={{ listStyle: "none", margin: 0, padding: 0 }}
                >
                  {LEGACY_ITEMS.map((item) => (
                    <li key={item} className="flex gap-3 text-sm text-muted leading-relaxed">
                      <span
                        className="font-mono font-bold flex-shrink-0"
                        style={{ color: "var(--danger)", minWidth: "1rem" }}
                      >
                        ✗
                      </span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
              <div className="nb-cell" style={{ borderLeft: "2px solid var(--amber)" }}>
                <div className="nb-label nb-label--amber">
                  VANTADB {mode === "rust" ? "Rust Core" : "Python SDK"} — {vantaLatency}ms
                </div>
                <ul
                  className="flex flex-col gap-3 mt-4"
                  style={{ listStyle: "none", margin: 0, padding: 0 }}
                >
                  {VANTA_ITEMS.map((item) => (
                    <li key={item} className="flex gap-3 text-sm text-foreground leading-relaxed">
                      <span className="font-mono font-bold flex-shrink-0 text-amber">✓</span>
                      {item}
                    </li>
                  ))}
                </ul>
                <div
                  className="mt-4 pt-3 font-mono text-[0.65rem] text-steel leading-relaxed tracking-[0.02em]"
                  style={{ borderTop: "1px solid var(--border)" }}
                >
                  <span className="text-amber">▲</span>{" "}
                  {mode === "rust"
                    ? "1.2ms p50 applies to Rust Core (native). Python SDK adds ~39.74ms p50 (FFI + serialization). Toggle above to compare."
                    : "Python SDK ~39.74ms p50 (~24.5% recall@10). Rust Core achieves 1.2ms p50 via zero-copy in-process access."}
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-label">02 / 03 — Pipeline Impact</div>
            <h2 className="font-display text-[clamp(1.5rem,3vw,2rem)] font-extrabold tracking-[-0.04em] mt-4 mb-12 leading-tight">
              Multi-query impact.
            </h2>

            <div className="nb-frame mb-0" data-frame-label="QUERY VOLUME">
              <div className="flex justify-between font-mono text-[0.65rem] text-steel uppercase tracking-[0.06em] mb-4">
                <span>
                  Queries in pipeline:{" "}
                  <span className="text-foreground font-bold">{pipelineSize}</span>
                </span>
                <span>Drag to adjust</span>
              </div>
              <input
                type="range"
                min={1}
                max={200}
                value={pipelineSize}
                onChange={(e) => setPipelineSize(Number(e.target.value))}
                style={{ width: "100%", accentColor: "var(--amber)", cursor: "pointer" }}
              />
            </div>

            <div className="nb-grid nb-grid--cols-3">
              <div className="nb-cell">
                <div className="nb-label" style={{ color: "var(--steel)" }}>
                  LEGACY
                </div>
                <div
                  className="font-display text-[2.5rem] font-extrabold tracking-[-0.05em] leading-none"
                  style={{ color: "var(--danger)" }}
                >
                  {(legacyTotal / 1000).toFixed(1)}s
                </div>
                <div className="font-mono text-[0.6rem] text-steel mt-2">
                  {pipelineSize} × 200ms
                </div>
              </div>
              <div className="nb-cell" style={{ borderLeft: "2px solid var(--amber)" }}>
                <div className="nb-label nb-label--amber">
                  VANTADB {mode === "rust" ? "RUST CORE" : "PYTHON SDK"}
                </div>
                <div className="font-display text-[2.5rem] font-extrabold tracking-[-0.05em] leading-none text-amber">
                  {vantaTotal < 1000
                    ? `${Math.round(vantaTotal)}ms`
                    : `${(vantaTotal / 1000).toFixed(1)}s`}
                </div>
                <div className="font-mono text-[0.6rem] text-steel mt-2">
                  {pipelineSize} × {vantaLatency}ms
                </div>
              </div>
              <div className="nb-cell flex flex-col justify-center items-center">
                <div className="font-display text-[3rem] font-extrabold tracking-[-0.05em] leading-none text-foreground">
                  {speedup}×
                </div>
                <div className="font-mono text-[0.6rem] text-amber uppercase tracking-[0.08em] mt-2">
                  Faster
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-label">03 / 03 — Where the Milliseconds Go</div>

            <div className="nb-frame mt-12" data-frame-label="BREAKDOWN">
              <div
                className="grid grid-cols-[120px_1fr_80px_80px] gap-4 px-4 py-3"
                style={{
                  borderBottom: "2px solid var(--border-visible)",
                  background: "var(--surface)",
                }}
              >
                <span className="font-mono text-[0.6rem] text-steel uppercase tracking-[0.08em]">
                  Phase
                </span>
                <span />
                <span className="font-mono text-[0.6rem] text-danger uppercase tracking-[0.06em] text-right">
                  Legacy
                </span>
                <span className="font-mono text-[0.6rem] text-amber uppercase tracking-[0.06em] text-right">
                  Vanta
                </span>
              </div>
              {BREAKDOWN.map((row, i) => (
                <div
                  key={row.label}
                  className="grid grid-cols-[120px_1fr_80px_80px] gap-4 px-4 py-5 items-center"
                  style={{
                    borderBottom: i < BREAKDOWN.length - 1 ? "1px solid var(--border)" : "none",
                  }}
                >
                  <span className="text-sm text-muted">{row.label}</span>
                  <div className="flex flex-col gap-[3px]">
                    <div className="h-[6px] relative" style={{ background: "var(--surface)" }}>
                      <div
                        className="absolute left-0 top-0 bottom-0"
                        style={{
                          width: `${row.legacyW}%`,
                          background: "var(--danger)",
                          opacity: 0.7,
                        }}
                      />
                    </div>
                    <div className="h-[6px] relative" style={{ background: "var(--surface)" }}>
                      <div
                        className="absolute left-0 top-0 bottom-0"
                        style={{
                          width: `${row.vantaW}%`,
                          background: "var(--amber)",
                          opacity: 0.9,
                        }}
                      />
                    </div>
                  </div>
                  <span className="font-mono text-[0.72rem] text-danger text-right">
                    {row.legacy}
                  </span>
                  <span className="font-mono text-[0.72rem] text-amber text-right">
                    {row.vanta}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="nb-section nb-bg-dot">
          <div className="nb-inner">
            <div className="nb-block-amber">
              <div className="flex items-center justify-between gap-4 flex-wrap">
                <div>
                  <div className="nb-label" style={{ color: "var(--text-on-amber)" }}>
                    GET STARTED
                  </div>
                  <h2
                    className="font-display text-2xl font-extrabold"
                    style={{ color: "var(--text-on-amber)" }}
                  >
                    1.2ms at the core. No network tax.
                  </h2>
                  <p className="text-sm" style={{ color: "var(--text-on-amber)", opacity: 0.8 }}>
                    Install VantaDB in one command.
                  </p>
                </div>
                <code
                  className="font-mono text-lg font-bold"
                  style={{ color: "var(--text-on-amber)" }}
                >
                  pip install vantadb-py
                </code>
              </div>
            </div>
          </div>
        </section>
      </main>

      <style>{`
        @media (max-width: 768px) {
          [style*="grid-template-columns: 120px 1fr 80px 80px"] { grid-template-columns: 1fr 1fr !important; gap: 0.5rem !important; }
        }
      `}</style>
    </div>
  );
}

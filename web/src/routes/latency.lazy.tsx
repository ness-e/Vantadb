import { createLazyRoute } from "@tanstack/react-router";
import { useRef, useState, useMemo } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/latency.css";

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
  "Network round-trip: 50\u201380ms (TLS + serialization)",
  "Remote index traversal: 40\u201360ms",
  "Result serialization: 20\u201330ms",
  "Cache miss penalty: 100ms+ to S3 fallback",
  "Cold start: 2\u20135s (serverless DB wake)",
];

const VANTA_ITEMS = [
  "Zero network: same-process memory access",
  "HNSW graph traversal: 0.4\u20130.8ms",
  "BM25 intersection: 0.2\u20130.4ms",
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

  const statsRef = useRef<HTMLElement>(null);
  const pipelineRef = useRef<HTMLElement>(null);
  const breakdownRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(statsRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, statsRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(pipelineRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, pipelineRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(breakdownRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, breakdownRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p06"
        title={
          <span>
            1.2ms p50 (Rust Core).
            <br />
            No network.
          </span>
        }
        sub="VantaDB runs in your process \u2014 no network round-trip, no serialization overhead, no cold starts. Every microsecond matters when your agent is waiting. Rust Core: 1.2ms p50 / Python SDK: ~39.74ms p50"
      />

      <div className="latency-toggle-bar">
        <button
          onClick={() => setMode("rust")}
          className={`latency-mode-btn ${mode === "rust" ? "latency-mode-btn--active" : ""}`}
        >
          Rust Core \u2014 1.2ms p50
        </button>
        <button
          onClick={() => setMode("python")}
          className={`latency-mode-btn ${mode === "python" ? "latency-mode-btn--active" : ""}`}
        >
          Python SDK \u2014 ~39.74ms p50
        </button>
      </div>

      <main>
        <NbSection ref={statsRef} ariaLabel="Latency comparison">
          <NbSectionHeader
            monoLabel="[LATENCY OVERVIEW]"
            headline="In-process vs network."
            sub="VantaDB eliminates every source of latency that traditional vector databases impose \u2014 no serialization, no network hops, no cold starts."
          />

          <div className="nb-grid nb-grid--cols-2">
            <div className="nb-cell nb-engine-part">
              <span className="latency-label-legacy">LEGACY \u2014 ~200ms</span>
              <ul className="latency-ul-reset latency-list-gap">
                {LEGACY_ITEMS.map((item) => (
                  <li key={item} className="latency-list-item">
                    <span className="latency-icon latency-icon--danger">\u2717</span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nb-cell latency-cell-border nb-engine-part">
              <span className="latency-label-vanta">
                VANTADB {mode === "rust" ? "Rust Core" : "Python SDK"} \u2014 {vantaLatency}ms
              </span>
              <ul className="latency-ul-reset latency-list-gap">
                {VANTA_ITEMS.map((item) => (
                  <li key={item} className="latency-list-item latency-list-item--vanta">
                    <span className="latency-icon latency-icon--vanta">\u2713</span>
                    {item}
                  </li>
                ))}
              </ul>
              <div className="latency-footnote">
                <span className="latency-footnote-icon">\u25B2</span>{" "}
                {mode === "rust"
                  ? "1.2ms p50 applies to Rust Core (native). Python SDK adds ~39.74ms p50 (FFI + serialization). Toggle above to compare."
                  : "Python SDK ~39.74ms p50 (~24.5% recall@10). Rust Core achieves 1.2ms p50 via zero-copy in-process access."}
              </div>
            </div>
          </div>
        </NbSection>

        <NbSection ref={pipelineRef} ariaLabel="Pipeline impact">
          <NbSectionHeader
            monoLabel="[PIPELINE IMPACT]"
            headline="Multi-query impact."
            sub="Drag the slider to simulate batch query pipelines and see how latency compounds \u2014 or doesn\u2019t."
          />

          <div className="nb-frame nb-engine-part">
            <div className="latency-slider-header">
              <span>
                Queries in pipeline: <span className="latency-slider-value">{pipelineSize}</span>
              </span>
              <span>Drag to adjust</span>
            </div>
            <input
              type="range"
              min={1}
              max={200}
              value={pipelineSize}
              onChange={(e) => setPipelineSize(Number(e.target.value))}
              className="latency-slider"
            />
          </div>

          <div className="nb-grid nb-grid--cols-3 nb-engine-part">
            <div className="nb-cell">
              <span className="latency-label-value">LEGACY</span>
              <div className="latency-value-lg latency-value-lg--danger">
                {(legacyTotal / 1000).toFixed(1)}s
              </div>
              <div className="latency-formula">{pipelineSize} \u00d7 200ms</div>
            </div>
            <div className="nb-cell latency-cell-border">
              <span className="latency-label-vanta">
                VANTADB {mode === "rust" ? "RUST CORE" : "PYTHON SDK"}
              </span>
              <div className="latency-value-lg latency-value-lg--amber">
                {vantaTotal < 1000
                  ? `${Math.round(vantaTotal)}ms`
                  : `${(vantaTotal / 1000).toFixed(1)}s`}
              </div>
              <div className="latency-formula">
                {pipelineSize} \u00d7 {vantaLatency}ms
              </div>
            </div>
            <div className="nb-cell latency-speedup-cell">
              <div className="latency-speedup-value">{speedup}\u00d7</div>
              <div className="latency-speedup-label">Faster</div>
            </div>
          </div>
        </NbSection>

        <NbSection ref={breakdownRef} ariaLabel="Latency breakdown">
          <NbSectionHeader
            monoLabel="[BREAKDOWN]"
            headline="Where the Milliseconds Go."
            sub="A granular look at where time is spent in a typical query \u2014 legacy vs VantaDB."
          />

          <div className="nb-frame nb-engine-part">
            <div className="latency-table-header">
              <span>Phase</span>
              <span />
              <span className="latency-table-col-header latency-table-col-header--danger">
                Legacy
              </span>
              <span className="latency-table-col-header latency-table-col-header--amber">
                Vanta
              </span>
            </div>
            {BREAKDOWN.map((row, i) => (
              <div
                key={row.label}
                className={`latency-table-row ${i < BREAKDOWN.length - 1 ? "latency-table-row--bordered" : ""}`}
              >
                <span className="latency-table-label">{row.label}</span>
                <div className="latency-bar-group">
                  <div className="latency-bar latency-bar--bg">
                    <div
                      className="latency-bar-fill latency-bar-fill--danger"
                      style={{ "--pct": `${row.legacyW}%` } as React.CSSProperties}
                    />
                  </div>
                  <div className="latency-bar latency-bar--bg">
                    <div
                      className="latency-bar-fill latency-bar-fill--amber"
                      style={{ "--pct": `${row.vantaW}%` } as React.CSSProperties}
                    />
                  </div>
                </div>
                <span className="latency-table-ms latency-table-ms--danger">{row.legacy}</span>
                <span className="latency-table-ms latency-table-ms--amber">{row.vanta}</span>
              </div>
            ))}
          </div>
        </NbSection>

        <NbSection variant="dark" className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="latency-cta-layout">
              <div>
                <h2 className="latency-cta-title">1.2ms at the core. No network tax.</h2>
                <p className="latency-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="latency-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}

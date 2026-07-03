import { createFileRoute } from "@tanstack/react-router";
import { useState, useMemo } from "react";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/latency")({
  head: () => ({
    meta: [
      { title: "VantaDB — Sub-Millisecond Latency" },
      {
        name: "description",
        content:
          "Rust Core 1.2ms p50 (Python SDK ~39.74ms p50) in-process latency vs 200ms+ for cloud vector databases. VantaDB eliminates network round-trips.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/latency" }],
  }),
  component: LatencyPage,
});

// ── Data ─────────────────────────────────────────────────────────────────────
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
  const vantaLatency = useMemo(() => mode === "rust" ? 1.2 : 39.74, [mode]);
  const legacyTotal = useMemo(() => pipelineSize * 200, [pipelineSize]);
  const vantaTotal = useMemo(() => pipelineSize * vantaLatency, [pipelineSize, vantaLatency]);
  const speedup = useMemo(() => Math.round(legacyTotal / Math.max(vantaTotal, 0.1)), [legacyTotal, vantaTotal]);

  return (
    <div className="engine-page">
      <SwissSubpageHero
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

      {/* Mode toggle */}
      <div style={{ display: "flex", justifyContent: "center", gap: "0.5rem", margin: "1.5rem 0 0" }}>
        <button
          onClick={() => setMode("rust")}
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "0.72rem",
            padding: "0.5rem 1.25rem",
            border: mode === "rust" ? "2px solid var(--amber)" : "1px solid var(--border)",
            background: mode === "rust" ? "var(--surface-raised)" : "transparent",
            color: mode === "rust" ? "var(--amber)" : "var(--muted)",
            cursor: "pointer",
            borderRadius: "0",
            letterSpacing: "0.04em",
          }}
        >
          Rust Core — 1.2ms p50
        </button>
        <button
          onClick={() => setMode("python")}
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "0.72rem",
            padding: "0.5rem 1.25rem",
            border: mode === "python" ? "2px solid var(--amber)" : "1px solid var(--border)",
            background: mode === "python" ? "var(--surface-raised)" : "transparent",
            color: mode === "python" ? "var(--amber)" : "var(--muted)",
            cursor: "pointer",
            borderRadius: "0",
            letterSpacing: "0.04em",
          }}
        >
          Python SDK — ~39.74ms p50
        </button>
      </div>

      <main className="engine-main">
        {/* Section 1: Comparison */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 03 — Comparison</span>
          <div
            className="swiss-grid-12"
            style={{ alignItems: "start", marginTop: "3rem", gap: "1px" }}
          >
            <div
              className="col-span-6"
              style={{ border: "1px solid var(--border)", padding: "2.5rem" }}
            >
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "1.1rem",
                  fontWeight: 800,
                  letterSpacing: "-0.03em",
                  color: "var(--steel)",
                  marginBottom: "2rem",
                  textTransform: "uppercase",
                }}
              >
                Legacy — ~200ms
              </h2>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.9rem",
                }}
              >
                {LEGACY_ITEMS.map((item, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.82rem",
                      color: "var(--muted)",
                      lineHeight: 1.5,
                    }}
                  >
                    <span
                      style={{
                        color: "#ff3b30",
                        fontWeight: 700,
                        minWidth: "1rem",
                        fontFamily: "var(--font-mono)",
                        flexShrink: 0,
                      }}
                    >
                      ✗
                    </span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div
              className="col-span-6"
              style={{
                border: "1px solid var(--border)",
                borderLeft: "2px solid var(--amber)",
                padding: "2.5rem",
                background: "var(--surface)",
              }}
            >
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "1.1rem",
                  fontWeight: 800,
                  letterSpacing: "-0.03em",
                  color: "var(--amber)",
                  marginBottom: "2rem",
                  textTransform: "uppercase",
                }}
              >
                VantaDB {mode === "rust" ? "Rust Core" : "Python SDK"} — {vantaLatency}ms
              </h2>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.9rem",
                }}
              >
                {VANTA_ITEMS.map((item, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.82rem",
                      color: "var(--foreground)",
                      lineHeight: 1.5,
                    }}
                  >
                    <span
                      style={{
                        color: "var(--amber)",
                        fontWeight: 700,
                        minWidth: "1rem",
                        fontFamily: "var(--font-mono)",
                        flexShrink: 0,
                      }}
                    >
                      ✓
                    </span>
                    {item}
                  </li>
                ))}
              </ul>
              <div
                style={{
                  marginTop: "1.5rem",
                  paddingTop: "1rem",
                  borderTop: "1px solid var(--border)",
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.65rem",
                  color: "var(--steel)",
                  lineHeight: 1.6,
                  letterSpacing: "0.02em",
                }}
              >
                <span style={{ color: "var(--amber)" }}>▲</span>{" "}
                {mode === "rust"
                  ? '1.2ms p50 applies to Rust Core (native). Python SDK adds ~39.74ms p50 (FFI + serialization). Toggle above to compare.'
                  : 'Python SDK ~39.74ms p50 (~24.5% recall@10). Rust Core achieves 1.2ms p50 via zero-copy in-process access.'}
              </div>
            </div>
          </div>
        </section>

        {/* Section 2: Pipeline Simulator */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">02 / 03 — Pipeline Impact</span>
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "clamp(1.5rem, 3vw, 2rem)",
              fontWeight: 800,
              letterSpacing: "-0.04em",
              margin: "1.25rem 0 3rem",
              lineHeight: 1.05,
            }}
          >
            Multi-query impact.
          </h2>

          <div
            style={{ border: "1px solid var(--border)", padding: "2.5rem", marginBottom: "1px" }}
          >
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                fontFamily: "var(--font-mono)",
                fontSize: "0.65rem",
                color: "var(--steel)",
                textTransform: "uppercase",
                letterSpacing: "0.06em",
                marginBottom: "1.5rem",
              }}
            >
              <span>
                Queries in pipeline:{" "}
                <span style={{ color: "var(--foreground)", fontWeight: 700 }}>{pipelineSize}</span>
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

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr 1fr",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
            }}
          >
            <div style={{ background: "var(--background)", padding: "2rem 2.5rem" }}>
              <div
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  color: "var(--steel)",
                  textTransform: "uppercase",
                  letterSpacing: "0.08em",
                  marginBottom: "0.5rem",
                }}
              >
                LEGACY
              </div>
              <div
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "2.5rem",
                  fontWeight: 800,
                  letterSpacing: "-0.05em",
                  color: "#ff3b30",
                  lineHeight: 1,
                }}
              >
                {(legacyTotal / 1000).toFixed(1)}s
              </div>
              <div
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  color: "var(--steel)",
                  marginTop: "0.5rem",
                }}
              >
                {pipelineSize} × 200ms
              </div>
            </div>
            <div
              style={{
                background: "var(--surface-raised)",
                padding: "2rem 2.5rem",
                borderLeft: "2px solid var(--amber)",
              }}
            >
              <div
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  color: "var(--amber)",
                  textTransform: "uppercase",
                  letterSpacing: "0.08em",
                  marginBottom: "0.5rem",
                }}
              >
                VANTADB {mode === "rust" ? "RUST CORE" : "PYTHON SDK"}
              </div>
              <div
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "2.5rem",
                  fontWeight: 800,
                  letterSpacing: "-0.05em",
                  color: "var(--amber)",
                  lineHeight: 1,
                }}
              >
                {vantaTotal < 1000
                  ? `${Math.round(vantaTotal)}ms`
                  : `${(vantaTotal / 1000).toFixed(1)}s`}
              </div>
              <div
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  color: "var(--steel)",
                  marginTop: "0.5rem",
                }}
              >
                {pipelineSize} × {vantaLatency}ms
              </div>
            </div>
            <div
              style={{
                background: "var(--background)",
                padding: "2rem 2.5rem",
                display: "flex",
                flexDirection: "column",
                justifyContent: "center",
                alignItems: "center",
              }}
            >
              <div
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "3rem",
                  fontWeight: 800,
                  letterSpacing: "-0.05em",
                  color: "var(--foreground)",
                  lineHeight: 1,
                }}
              >
                {speedup}×
              </div>
              <div
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  color: "var(--amber)",
                  textTransform: "uppercase",
                  letterSpacing: "0.08em",
                  marginTop: "0.5rem",
                }}
              >
                Faster
              </div>
            </div>
          </div>
        </section>

        {/* Section 3: Breakdown bars */}
        <section className="engine-section">
          <span className="swiss-eyebrow">03 / 03 — Where the Milliseconds Go</span>
          <div style={{ border: "1px solid var(--border)", marginTop: "3rem" }}>
            {/* Header */}
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "120px 1fr 80px 80px",
                gap: "1rem",
                padding: "0.75rem 1.5rem",
                borderBottom: "2px solid var(--border)",
                background: "var(--surface)",
              }}
            >
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  color: "var(--steel)",
                  textTransform: "uppercase",
                  letterSpacing: "0.08em",
                }}
              >
                Phase
              </span>
              <span />
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  color: "#ff3b30",
                  textTransform: "uppercase",
                  letterSpacing: "0.06em",
                  textAlign: "right",
                }}
              >
                Legacy
              </span>
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  color: "var(--amber)",
                  textTransform: "uppercase",
                  letterSpacing: "0.06em",
                  textAlign: "right",
                }}
              >
                Vanta
              </span>
            </div>
            {BREAKDOWN.map((row, i) => (
              <div
                key={row.label}
                style={{
                  display: "grid",
                  gridTemplateColumns: "120px 1fr 80px 80px",
                  gap: "1rem",
                  padding: "1.25rem 1.5rem",
                  borderBottom: i < BREAKDOWN.length - 1 ? "1px solid var(--border)" : "none",
                  alignItems: "center",
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.8rem",
                    color: "var(--muted)",
                  }}
                >
                  {row.label}
                </span>
                <div style={{ display: "flex", flexDirection: "column", gap: "3px" }}>
                  <div
                    style={{ height: "6px", background: "var(--surface)", position: "relative" }}
                  >
                    <div
                      style={{
                        position: "absolute",
                        left: 0,
                        top: 0,
                        bottom: 0,
                        width: `${row.legacyW}%`,
                        background: "#ff3b30",
                        opacity: 0.7,
                      }}
                    />
                  </div>
                  <div
                    style={{ height: "6px", background: "var(--surface)", position: "relative" }}
                  >
                    <div
                      style={{
                        position: "absolute",
                        left: 0,
                        top: 0,
                        bottom: 0,
                        width: `${row.vantaW}%`,
                        background: "var(--amber)",
                        opacity: 0.9,
                      }}
                    />
                  </div>
                </div>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.72rem",
                    color: "#ff3b30",
                    textAlign: "right",
                  }}
                >
                  {row.legacy}
                </span>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.72rem",
                    color: "var(--amber)",
                    textAlign: "right",
                  }}
                >
                  {row.vanta}
                </span>
              </div>
            ))}
          </div>
        </section>
      </main>
    </div>
  );
}

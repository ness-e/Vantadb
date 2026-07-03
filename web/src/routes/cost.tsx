import { createFileRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/cost")({
  head: () => ({
    meta: [
      { title: "VantaDB — Infrastructure Cost: Zero Runtime" },
      {
        name: "description",
        content:
          "Eliminate $200+/mo infrastructure costs. VantaDB runs in-process with zero cloud dependencies and no per-query pricing.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/cost" }],
  }),
  component: CostPage,
});

// ── Data ─────────────────────────────────────────────────────────────────────
const PROVIDERS = [
  {
    name: "Pinecone + Redis + S3",
    total: 200,
    breakdown: { "Vector DB": 70, Cache: 30, Storage: 15, Egress: 25, Ops: 60 },
  },
  {
    name: "Weaviate Cloud",
    total: 175,
    breakdown: { "Vector DB": 90, Cache: 0, Storage: 20, Egress: 15, Ops: 50 },
  },
  {
    name: "Qdrant Cloud",
    total: 163,
    breakdown: { "Vector DB": 80, Cache: 0, Storage: 18, Egress: 20, Ops: 45 },
  },
  {
    name: "VantaDB",
    total: 0,
    breakdown: { "Vector DB": 0, Cache: 0, Storage: 0, Egress: 0, Ops: 0 },
  },
];

const LEGACY_COSTS = [
  "Pinecone: $70/mo (pod-based, 1M vectors)",
  "Redis: $30/mo (ElastiCache serverless)",
  "S3: $15/mo + API request costs",
  "Network egress: unpredictable overage fees",
  "Ops overhead: monitoring, scaling, patching",
];

const VANTA_COSTS = [
  "Free and open-source (Apache 2.0)",
  "No cloud dependency — runs on your hardware",
  "No per-query or per-vector pricing",
  "Zero ops cost — no servers to maintain",
];

function CostPage() {
  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="08"
        eyebrow="Infrastructure Cost"
        title={
          <span>
            Zero cost
            <br />
            at runtime.
          </span>
        }
        sub="No per-vector pricing, no server bills, no hidden egress fees. VantaDB is free software — the only cost is the hardware you already own."
      />

      <main className="engine-main">
        {/* Section 1: Comparison */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 02 — Cost Comparison</span>
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
                Legacy — ~$200/mo
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
                {LEGACY_COSTS.map((item, i) => (
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
                VantaDB — $0
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
                {VANTA_COSTS.map((item, i) => (
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
            </div>
          </div>
        </section>

        {/* Section 2: Cost breakdown table */}
        <section className="engine-section">
          <span className="swiss-eyebrow">02 / 02 — Monthly Cost by Provider</span>
          <div style={{ border: "1px solid var(--border)", marginTop: "3rem", overflowX: "auto" }}>
            <table
              style={{
                width: "100%",
                borderCollapse: "collapse",
                fontFamily: "var(--font-sans)",
                fontSize: "0.8rem",
              }}
            >
              <thead>
                <tr style={{ borderBottom: "2px solid var(--border)" }}>
                  <th
                    style={{
                      padding: "1rem 1.5rem",
                      textAlign: "left",
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.6rem",
                      textTransform: "uppercase",
                      letterSpacing: "0.08em",
                      color: "var(--steel)",
                      fontWeight: 600,
                    }}
                  >
                    Component
                  </th>
                  {PROVIDERS.map((p) => (
                    <th
                      key={p.name}
                      style={{
                        padding: "1rem 1.5rem",
                        textAlign: "right",
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.6rem",
                        textTransform: "uppercase",
                        letterSpacing: "0.06em",
                        color: p.total === 0 ? "var(--amber)" : "var(--steel)",
                        fontWeight: 600,
                      }}
                    >
                      {p.name}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {["Vector DB", "Cache", "Storage", "Egress", "Ops"].map((comp, i) => (
                  <tr
                    key={comp}
                    style={{
                      borderBottom: "1px solid var(--border)",
                      background: i % 2 === 0 ? "var(--background)" : "var(--surface)",
                    }}
                  >
                    <td style={{ padding: "0.9rem 1.5rem", color: "var(--muted)" }}>{comp}</td>
                    {PROVIDERS.map((p) => {
                      const val = p.breakdown[comp as keyof typeof p.breakdown];
                      return (
                        <td
                          key={p.name}
                          style={{
                            padding: "0.9rem 1.5rem",
                            textAlign: "right",
                            fontFamily: "var(--font-mono)",
                            fontSize: "0.75rem",
                            color: val === 0 ? "var(--amber)" : "var(--foreground)",
                          }}
                        >
                          ${val}
                        </td>
                      );
                    })}
                  </tr>
                ))}
                <tr
                  style={{
                    borderTop: "2px solid var(--border)",
                    background: "var(--surface-raised)",
                  }}
                >
                  <td
                    style={{
                      padding: "1rem 1.5rem",
                      fontFamily: "var(--font-display)",
                      fontWeight: 800,
                      color: "var(--foreground)",
                    }}
                  >
                    Total
                  </td>
                  {PROVIDERS.map((p) => (
                    <td
                      key={p.name}
                      style={{
                        padding: "1rem 1.5rem",
                        textAlign: "right",
                        fontFamily: "var(--font-display)",
                        fontWeight: 800,
                        fontSize: "1rem",
                        color: p.total === 0 ? "var(--amber)" : "var(--foreground)",
                      }}
                    >
                      ${p.total}
                    </td>
                  ))}
                </tr>
              </tbody>
            </table>
          </div>

          {/* TCO note */}
          <div
            style={{
              border: "1px solid var(--border)",
              borderTop: "none",
              background: "var(--surface)",
              padding: "1.5rem 2.5rem",
              display: "grid",
              gridTemplateColumns: "140px 1fr",
              gap: "2rem",
              alignItems: "center",
            }}
          >
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "0.6rem",
                color: "var(--amber)",
                textTransform: "uppercase",
                letterSpacing: "0.08em",
              }}
            >
              TCO NOTE
            </span>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "0.8rem",
                color: "var(--muted)",
                lineHeight: 1.6,
                margin: 0,
              }}
            >
              By eliminating three managed services, VantaDB removes the single largest variable
              cost from your vector search infrastructure. Your only expense is the compute you
              already run.
            </p>
          </div>
        </section>
      </main>
    </div>
  );
}

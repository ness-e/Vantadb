import { createLazyRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/cost")({
  component: CostPage,
  pendingComponent: PendingComponent,
});

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
    <div>
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

      <main>
        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-label">01 / 02 — Cost Comparison</div>

            <div className="nb-grid nb-grid--cols-2" style={{ marginTop: "3rem" }}>
              <div className="nb-cell">
                <div className="nb-label" style={{ color: "var(--steel)" }}>
                  LEGACY — ~$200/mo
                </div>
                <ul
                  className="flex flex-col gap-3 mt-4"
                  style={{ listStyle: "none", margin: 0, padding: 0 }}
                >
                  {LEGACY_COSTS.map((item) => (
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
                <div className="nb-label nb-label--amber">VANTADB — $0</div>
                <ul
                  className="flex flex-col gap-3 mt-4"
                  style={{ listStyle: "none", margin: 0, padding: 0 }}
                >
                  {VANTA_COSTS.map((item) => (
                    <li key={item} className="flex gap-3 text-sm text-foreground leading-relaxed">
                      <span
                        className="font-mono font-bold flex-shrink-0"
                        style={{ color: "var(--amber)", minWidth: "1rem" }}
                      >
                        ✓
                      </span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        </section>

        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-label">02 / 02 — Monthly Cost by Provider</div>

            <div className="nb-frame mt-12" data-frame-label="COST TABLE">
              <table className="nb-table" style={{ border: "none" }}>
                <thead>
                  <tr>
                    <th>Component</th>
                    {PROVIDERS.map((p) => (
                      <th
                        key={p.name}
                        className="text-right"
                        style={{ color: p.total === 0 ? "var(--amber)" : undefined }}
                      >
                        {p.name}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {(["Vector DB", "Cache", "Storage", "Egress", "Ops"] as const).map((comp) => (
                    <tr key={comp}>
                      <td style={{ color: "var(--muted)" }}>{comp}</td>
                      {PROVIDERS.map((p) => {
                        const val = p.breakdown[comp];
                        return (
                          <td
                            key={p.name}
                            className="text-right font-mono text-[0.75rem]"
                            style={{ color: val === 0 ? "var(--amber)" : "var(--foreground)" }}
                          >
                            ${val}
                          </td>
                        );
                      })}
                    </tr>
                  ))}
                </tbody>
                <tfoot>
                  <tr style={{ borderTop: "2px solid var(--border-visible)" }}>
                    <td className="font-display font-extrabold">Total</td>
                    {PROVIDERS.map((p) => (
                      <td
                        key={p.name}
                        className="text-right font-display font-extrabold text-base"
                        style={{ color: p.total === 0 ? "var(--amber)" : "var(--foreground)" }}
                      >
                        ${p.total}
                      </td>
                    ))}
                  </tr>
                </tfoot>
              </table>
            </div>

            <div className="nb-grid nb-grid--cols-2" style={{ marginTop: "0" }}>
              <div
                className="nb-cell"
                style={{
                  gridColumn: "1 / -1",
                  display: "grid",
                  gridTemplateColumns: "140px 1fr",
                  gap: "2rem",
                  alignItems: "center",
                }}
              >
                <span className="font-mono text-[0.6rem] text-amber uppercase tracking-[0.08em]">
                  TCO NOTE
                </span>
                <p className="text-sm text-muted leading-relaxed m-0">
                  By eliminating three managed services, VantaDB removes the single largest variable
                  cost from your vector search infrastructure. Your only expense is the compute you
                  already run.
                </p>
              </div>
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
                    Free software. Zero runtime cost.
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
    </div>
  );
}

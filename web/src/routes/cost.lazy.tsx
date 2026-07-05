import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/cost.css";

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
      <NbSubpageHero
        num="08"
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
            <h2 className="cost-section-title">
              Cost Comparison
            </h2>

            <div className="nb-grid nb-grid--cols-2 cost-grid">
              <div className="nb-cell">
                <div className="cost-label-legacy">
                  LEGACY — ~$200/mo
                </div>
                <ul
                  className="flex flex-col gap-3 mt-4 cost-list"
                >
                  {LEGACY_COSTS.map((item) => (
                    <li key={item} className="flex gap-3 text-sm text-muted leading-relaxed">
                      <span
                        className="font-mono font-bold flex-shrink-0 cost-icon-danger"
                      >
                        ✗
                      </span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
              <div className="nb-cell cost-cell-border">
                <div className="cost-label-vanta">
                  VANTADB — $0
                </div>
                <ul
                  className="flex flex-col gap-3 mt-4 cost-list"
                >
                  {VANTA_COSTS.map((item) => (
                    <li key={item} className="flex gap-3 text-sm text-foreground leading-relaxed">
                      <span
                        className="font-mono font-bold flex-shrink-0 cost-icon-amber"
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
            <h2 className="cost-section-title">
              Monthly Cost by Provider
            </h2>

            <div className="nb-frame mt-12">
              <table className="nb-table cost-table">
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
                      <td className="cost-muted-cell">{comp}</td>
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
                  <tr className="cost-tfoot-row">
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

            <div className="nb-grid nb-grid--cols-2 cost-grid-flush">
              <div
                className="nb-cell cost-tco-cell"
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
                  <h2
                    className="font-display text-2xl font-extrabold cost-cta-title"
                  >
                    Free software. Zero runtime cost.
                  </h2>
                  <p className="text-sm cost-cta-sub">
                    Install VantaDB in one command.
                  </p>
                </div>
                <code
                  className="font-mono text-lg font-bold cost-cta-code"
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

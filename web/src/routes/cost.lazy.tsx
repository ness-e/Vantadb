import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbBlockAmber, NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
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
  const costRef = useRef<HTMLElement>(null);
  const providersRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = costRef.current?.querySelectorAll<HTMLElement>(".nc-cost-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, costRef);

  useAnimationSafe(() => {
    const parts = providersRef.current?.querySelectorAll<HTMLElement>(".nc-cost-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, providersRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p09"
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
        <NbSection ref={costRef} ariaLabel="Cost comparison">
          <NbSectionHeader
            monoLabel="[BREAKDOWN]"
            headline="Monthly cost comparison."
            sub="VantaDB eliminates the single largest variable cost from your vector search infrastructure — your only expense is the hardware you already run."
          />

          <div className="nc-cost-grid nc-cost-part">
            <div className="nc-cost-col">
              <span className="nc-cost-col-title nc-cost-col-title--danger">LEGACY — ~$200/mo</span>
              <ul className="nc-cost-list">
                {LEGACY_COSTS.map((item) => (
                  <li key={item} className="nc-cost-item">
                    <span className="nc-cost-icon nc-cost-icon--danger">✗</span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nc-cost-col nc-cost-col--vanta">
              <span className="nc-cost-col-title nc-cost-col-title--amber">VANTADB — $0</span>
              <ul className="nc-cost-list">
                {VANTA_COSTS.map((item) => (
                  <li key={item} className="nc-cost-item nc-cost-item--fg">
                    <span className="nc-cost-icon nc-cost-icon--amber">✓</span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </NbSection>

        <NbSection ref={providersRef} ariaLabel="Provider cost table">
          <NbSectionHeader
            monoLabel="[PROVIDERS]"
            headline="Cost by provider."
            sub="How VantaDB's $0 model stacks up against the leading managed vector database solutions."
          />

          <div className="nc-cost-part">
            <div style={{ overflowX: "auto" }}>
              <table className="nc-cost-provider-table">
                <thead>
                  <tr>
                    <th>Component</th>
                    {PROVIDERS.map((p) => (
                      <th key={p.name} data-amber={p.total === 0}>
                        {p.name}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {(["Vector DB", "Cache", "Storage", "Egress", "Ops"] as const).map((comp) => (
                    <tr key={comp}>
                      <td>{comp}</td>
                      {PROVIDERS.map((p) => {
                        const val = p.breakdown[comp];
                        return (
                          <td key={p.name} data-amber={val === 0}>
                            ${val}
                          </td>
                        );
                      })}
                    </tr>
                  ))}
                </tbody>
                <tfoot>
                  <tr>
                    <td>Total</td>
                    {PROVIDERS.map((p) => (
                      <td key={p.name} data-amber={p.total === 0}>
                        ${p.total}
                      </td>
                    ))}
                  </tr>
                </tfoot>
              </table>
            </div>
          </div>

          <div className="nc-cost-tco nc-cost-part">
            <span className="nc-cost-tco-label">TCO NOTE</span>
            <p className="nc-cost-tco-text">
              By eliminating three managed services, VantaDB removes the single largest variable
              cost from your vector search infrastructure. Your only expense is the compute you
              already run.
            </p>
          </div>
        </NbSection>

        <NbSection className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="nc-cost-cta">
              <div>
                <h2 className="nc-cost-cta-heading">Free software. Zero runtime cost.</h2>
                <p className="nc-cost-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="nc-cost-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}

import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
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
  const costRef = useRef<HTMLElement>(null);
  const providersRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(costRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, costRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(providersRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
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

          <div className="nb-engine-part">
            <div className="nb-grid nb-grid--cols-2 cost-grid">
              <div className="nb-cell">
                <div className="cost-label-legacy">LEGACY — ~$200/mo</div>
                <ul className="cost-list">
                  {LEGACY_COSTS.map((item) => (
                    <li key={item} className="cost-list-item">
                      <span className="cost-icon-danger">✗</span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
              <div className="nb-cell cost-cell-border">
                <div className="cost-label-vanta">VANTADB — $0</div>
                <ul className="cost-list">
                  {VANTA_COSTS.map((item) => (
                    <li key={item} className="cost-list-item cost-list-item--foreground">
                      <span className="cost-icon-amber">✓</span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        </NbSection>

        <NbSection ref={providersRef} ariaLabel="Provider cost table">
          <NbSectionHeader
            monoLabel="[PROVIDERS]"
            headline="Cost by provider."
            sub="How VantaDB's $0 model stacks up against the leading managed vector database solutions."
          />

          <div className="nb-engine-part">
            <div className="nb-card-frame">
              <table className="nb-table cost-table">
                <thead>
                  <tr>
                    <th>Component</th>
                    {PROVIDERS.map((p) => (
                      <th key={p.name} className="cost-th" data-amber={p.total === 0}>
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
                          <td key={p.name} className="cost-td" data-amber={val === 0}>
                            ${val}
                          </td>
                        );
                      })}
                    </tr>
                  ))}
                </tbody>
                <tfoot>
                  <tr className="cost-tfoot-row">
                    <td>Total</td>
                    {PROVIDERS.map((p) => (
                      <td key={p.name} className="cost-tfoot-value" data-amber={p.total === 0}>
                        ${p.total}
                      </td>
                    ))}
                  </tr>
                </tfoot>
              </table>
            </div>
          </div>

          <div className="nb-engine-part">
            <div className="nb-grid nb-grid--cols-2 cost-grid-flush">
              <div className="nb-cell cost-tco-cell">
                <span className="nb-mono-label">TCO NOTE</span>
                <p className="cost-tco-text">
                  By eliminating three managed services, VantaDB removes the single largest variable
                  cost from your vector search infrastructure. Your only expense is the compute you
                  already run.
                </p>
              </div>
            </div>
          </div>
        </NbSection>

        <NbSection className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="cost-cta-row">
              <div>
                <h2 className="cost-cta-heading">Free software. Zero runtime cost.</h2>
                <p className="cost-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="cost-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}

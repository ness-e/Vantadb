import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { PricingCard } from "@/components/PricingCard";
import { PricingCTA } from "@/components/PricingCTA";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import { tiers, comparisonColumns, comparisonRows, FAQ_ITEMS } from "@/data/pricing";
import "../styles/pricing.css";

export const Route = createLazyRoute("/pricing")({
  component: PricingPage,
  pendingComponent: PendingComponent,
});



function PricingPage() {
  const plansRef = useRef<HTMLElement>(null);
  const compareRef = useRef<HTMLElement>(null);
  const faqRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = plansRef.current?.querySelectorAll<HTMLElement>(".nc-price-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, plansRef);

  useAnimationSafe(() => {
    const parts = compareRef.current?.querySelectorAll<HTMLElement>(".nc-price-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, compareRef);

  useAnimationSafe(() => {
    const parts = faqRef.current?.querySelectorAll<HTMLElement>(".nc-price-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, faqRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p11"
        title={
          <span>
            Free to build.
            <br />
            Fair to scale.
          </span>
        }
        sub="VantaDB is open source (Apache 2.0) and free forever. Sign up for cloud databases to scale in production with SLAs, team features, and zero ops."
      />

      <main>
        <NbSection ref={plansRef} ariaLabel="Pricing plans">
          <NbSectionHeader
            monoLabel="[PLANS]"
            headline="Four tiers. One free."
            sub="Self-hosted is free forever under Apache 2.0. Cloud plans unlock managed infrastructure for teams that need it."
          />

          <div className="nc-price-board nc-price-part">
            {tiers.slice(0, 2).map((tier) => (
              <PricingCard key={tier.name} tier={tier} featured={tier.featured} />
            ))}
          </div>

          <div className="nc-price-board nc-price-board--tight nc-price-part">
            {tiers.slice(2).map((tier) => (
              <PricingCard key={tier.name} tier={tier} />
            ))}
          </div>
        </NbSection>

        <NbSection ref={compareRef} className="nb-bg-cross--faint" ariaLabel="Feature comparison">
          <NbSectionHeader
            monoLabel="[COMPARE]"
            headline="Side-by-side feature comparison."
            sub="Compare capabilities across all four tiers to find the right fit for your project."
          />

          <div className="nc-price-market nc-price-part">
            <table>
              <thead>
                <tr>
                  {comparisonColumns.map((col) => (
                    <th scope="col" key={col}>
                      {col}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {comparisonRows.map((row) => (
                  <tr key={row.feature}>
                    <td>{row.feature}</td>
                    <td>{row.os}</td>
                    <td>{row.pro}</td>
                    <td>{row.biz}</td>
                    <td>{row.ent}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </NbSection>

        <NbSection ref={faqRef} ariaLabel="FAQ">
          <NbSectionHeader
            monoLabel="[FAQ]"
            headline="Common questions."
            sub="Everything you need to know about VantaDB pricing, licensing, and cloud plans."
          />

          <div className="nc-price-faq nc-price-part">
            {FAQ_ITEMS.map((item) => (
              <div key={item.q} className="nc-price-faq-item">
                <h3 className="nc-price-faq-q">{item.q}</h3>
                <p className="nc-price-faq-a">{item.a}</p>
              </div>
            ))}
          </div>
        </NbSection>

        <NbSection ariaLabel="Get started">
          <PricingCTA
            heading="Start with Self-Hosted."
            sub="Free forever. No signup required."
            cta="GET STARTED"
            href="/docs"
          />
        </NbSection>
      </main>
    </div>
  );
}

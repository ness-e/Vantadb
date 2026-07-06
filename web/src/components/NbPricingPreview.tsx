import { NbSection, NbSectionHeader, NbButton } from "./nb";
import "../styles/pricing-preview.css";

const TIERS = [
  {
    id: "free", name: "FREE", tag: "MIT License", price: "$0",
    specs: [
      { code: "NODES", val: "1", note: "Single-node" },
      { code: "VECTORS", val: "10M", note: "Max vectors" },
      { code: "SUPPORT", val: "COM", note: "Community support" },
      { code: "FEATURES", val: "ALL", note: "All core features" },
    ],
    cta: "GET STARTED",
    ctaHref: "/docs/quickstart",
  },
  {
    id: "enterprise", name: "ENTERPRISE", tag: "Custom SLA", price: "Custom",
    specs: [
      { code: "DEPLOY", val: "ON-PREM", note: "On-prem deploy" },
      { code: "AUTH", val: "SSO", note: "SSO / SAML" },
      { code: "AUDIT", val: "YES", note: "Audit trails" },
      { code: "SLA", val: "4H", note: "Priority SLA" },
    ],
    cta: "CONTACT SALES",
    ctaHref: "/about/contact",
  },
];

export function NbPricingPreview() {
  return (
    <NbSection variant="lg" ariaLabel="Pricing">
      <NbSectionHeader
        monoLabel="[SPEC SHEET]"
        headline="Pricing."
        sub="Two tiers. No surprises. Open source at heart."
      />

      <div className="nb-spec-grid">
        {TIERS.map((tier) => (
          <div key={tier.id} className={`nb-spec-card nb-spec-card--${tier.id}`}>
            {/* Header */}
            <div className="nb-spec-header">
              <div className="nb-spec-header-top">
                <span className="nb-spec-tag">{tier.tag}</span>
                <span className="nb-spec-name">{tier.name}</span>
              </div>
              <span className="nb-spec-price">{tier.price}</span>
            </div>

            {/* Spec table */}
            <div className="nb-spec-table">
              <div className="nb-spec-table-head">
                <span>PARAMETER</span>
                <span>VALUE</span>
                <span>NOTE</span>
              </div>
              {tier.specs.map((s) => (
                <div key={s.code} className="nb-spec-row">
                  <span className="nb-spec-code">{s.code}</span>
                  <span className="nb-spec-val">{s.val}</span>
                  <span className="nb-spec-note">{s.note}</span>
                </div>
              ))}
            </div>

            {/* CTA */}
            <NbButton
              href={tier.ctaHref}
              variant={tier.id === "free" ? "primary" : "ghost"}
              className="nb-spec-btn"
            >
              {tier.cta}
            </NbButton>
          </div>
        ))}
      </div>
    </NbSection>
  );
}

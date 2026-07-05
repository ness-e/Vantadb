import { NbSection, NbSectionHeader, NbButton } from "./nb";
import "../styles/pricing-preview.css";

const FEATURES_FREE = ["Single-node", "10M vectors", "Community support", "All core features"];

const FEATURES_ENTERPRISE = ["On-prem deploy", "SSO / SAML", "Audit trails", "Priority SLA (4h)"];

export function NbPricingPreview() {
  return (
    <NbSection variant="lg" ariaLabel="Pricing">
      <NbSectionHeader monoLabel="[PRICING]" headline="Simple pricing." />

      <div className="nb-price-grid">
        <div className="nb-price-card nb-price-card--free">
          <div className="nb-price-card-header">
            <span className="nb-mono-label">FREE</span>
            <span className="nb-price-amount">$0</span>
          </div>
          <div className="nb-hairline nb-hairline--strong" />
          <ul className="nb-price-features">
            {FEATURES_FREE.map((f, i) => (
              <li key={f}>
                <span className="nb-price-feature-num">{String(i + 1).padStart(2, "0")}</span>
                <span>{f}</span>
              </li>
            ))}
          </ul>
          <NbButton href="/docs/quickstart" className="nb-price-btn">
            GET STARTED
          </NbButton>
        </div>

        <div className="nb-price-card nb-card-frame--featured nb-price-card--enterprise">
          <div className="nb-price-card-header">
            <span className="nb-mono-label">ENTERPRISE</span>
            <span className="nb-price-amount">Custom</span>
          </div>
          <div className="nb-hairline nb-hairline--strong" />
          <ul className="nb-price-features">
            {FEATURES_ENTERPRISE.map((f, i) => (
              <li key={f}>
                <span className="nb-price-feature-num nb-price-feature-num--amber">
                  {String(i + 1).padStart(2, "0")}
                </span>
                <span>{f}</span>
              </li>
            ))}
          </ul>
          <NbButton variant="ghost" href="/about/contact" className="nb-price-btn">
            CONTACT SALES
          </NbButton>
        </div>
      </div>
    </NbSection>
  );
}

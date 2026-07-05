import { Link } from "@tanstack/react-router";
import "../styles/pricing-preview.css";

const FEATURES_FREE = ["Single-node", "10M vectors", "Community support", "All core features"];

const FEATURES_ENTERPRISE = ["On-prem deploy", "SSO / SAML", "Audit trails", "Priority SLA (4h)"];

export function NbPricingPreview() {
  return (
    <section className="nb-section nb-section--lg" aria-label="Pricing">
      <div className="nb-inner">
        <span className="nb-mono-label">[PRICING]</span>
        <h2 className="nb-section-headline">Simple pricing.</h2>

        <div className="nb-price-grid">
          <div className="nb-price-card">
            <span className="nb-mono-label">FREE</span>
            <div className="nb-price-amount">$0</div>
            <div className="nb-hairline nb-hairline--strong" />
            <ul className="nb-price-features">
              {FEATURES_FREE.map((f) => (
                <li key={f}>{f}</li>
              ))}
            </ul>
            <Link to="/docs/quickstart" className="nb-btn">
              GET STARTED
            </Link>
          </div>

          <div className="nb-price-card nb-card-frame--featured">
            <span className="nb-mono-label">ENTERPRISE</span>
            <div className="nb-price-amount">Custom</div>
            <div className="nb-hairline nb-hairline--strong" />
            <ul className="nb-price-features">
              {FEATURES_ENTERPRISE.map((f) => (
                <li key={f}>{f}</li>
              ))}
            </ul>
            <Link to="/about/contact" className="nb-btn">
              CONTACT SALES
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}

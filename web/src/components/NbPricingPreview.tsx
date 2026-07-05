import { Link } from "@tanstack/react-router";
import "../styles/pricing-preview.css";

const FEATURES_FREE = ["Single-node", "10M vectors", "Community support", "All core features"];

const FEATURES_ENTERPRISE = ["On-prem deploy", "SSO / SAML", "Audit trails", "Priority SLA (4h)"];

export function NbPricingPreview() {
  return (
    <section className="nb-section" aria-label="Pricing">
      <div className="nb-inner">
        <h2 className="nb-amber-title">Pricing</h2>

        <hr className="nb-divider" />

        <div className="nb-pricing-grid">
          <div className="nb-pricing-card nb-pricing-card--free">
            <h3 className="nb-pricing-title nb-pricing-title--amber">FREE</h3>

            <div className="nb-pricing-price-row">
              <span className="nb-pricing-amount nb-pricing-amount--amber">$0</span>
              <span className="nb-pricing-label">forever</span>
            </div>

            <ul className="nb-pricing-features">
              {FEATURES_FREE.map((f) => (
                <li key={f} className="nb-pricing-feature">
                  {f}
                </li>
              ))}
            </ul>

            <Link to="/docs/quickstart" className="nb-pricing-cta nb-pricing-cta--free">
              GET STARTED
            </Link>
          </div>

          <div className="nb-pricing-card">
            <h3 className="nb-pricing-title nb-pricing-title--default">ENTERPRISE</h3>

            <div className="nb-pricing-price-row">
              <span className="nb-pricing-amount nb-pricing-amount--default">Custom</span>
              <span className="nb-pricing-label">tailored</span>
            </div>

            <ul className="nb-pricing-features">
              {FEATURES_ENTERPRISE.map((f) => (
                <li key={f} className="nb-pricing-feature">
                  {f}
                </li>
              ))}
            </ul>

            <Link to="/about/contact" className="nb-pricing-cta nb-pricing-cta--enterprise">
              CONTACT SALES
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}

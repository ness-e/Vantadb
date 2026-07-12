import { Link } from "@tanstack/react-router";
import { PricingFeatureRow } from "./PricingFeatureRow";

export interface PricingCardTier {
  name: string;
  tagline: string;
  price: string;
  period: string;
  features: string[];
  cta: string;
  href: string;
}

interface PricingCardProps {
  tier: PricingCardTier;
  featured?: boolean;
}

export function PricingCard({ tier, featured }: PricingCardProps) {
  return (
    <div className={`nc-price-card${featured ? " nc-price-card--featured" : ""}`}>
      <div>
        <div className="nc-price-name">{tier.name}</div>
        <p className="nc-price-tagline">{tier.tagline}</p>
      </div>
      <div className="nc-price-row">
        <span className="nc-price-value">{tier.price}</span>
        <span className="nc-price-period">{tier.period}</span>
      </div>
      <ul className="nc-price-features">
        {tier.features.map((f) => (
          <PricingFeatureRow key={f} feature={f} />
        ))}
      </ul>
      <Link
        to={tier.href.startsWith("/") ? (tier.href as "/") : "/about/contact"}
        className={`nc-price-cta nb-btn${featured ? "" : " nb-btn--ghost"}`}
      >
        {tier.cta}
      </Link>
    </div>
  );
}

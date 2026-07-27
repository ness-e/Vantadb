import type { ReactNode } from "react";

interface NbPricingCardProps {
  children: ReactNode;
  featured?: boolean;
  featuredLabel?: string;
  className?: string;
}

export function NbPricingCard({
  children,
  featured,
  featuredLabel = "POPULAR",
  className = "",
}: NbPricingCardProps) {
  return (
    <div className={`nb-pricing-card${featured ? " nb-pricing-card--featured" : ""} ${className}`}>
      {featured && <span className="nb-pricing-badge">{featuredLabel}</span>}
      {children}
    </div>
  );
}

export function NbPricingPrice({ children }: { children: ReactNode }) {
  return <div className="nb-pricing-price">{children}</div>;
}

export function NbPricingPeriod({ children }: { children: ReactNode }) {
  return <span className="nb-pricing-period">{children}</span>;
}

interface NbPricingFeatureProps {
  children: ReactNode;
  available?: boolean;
}

export function NbPricingFeature({ children, available = true }: NbPricingFeatureProps) {
  return <li className={available ? "" : "nb-pricing-feature--na"}>{children}</li>;
}

export function NbPricingFeatures({ children }: { children: ReactNode }) {
  return <ul className="nb-pricing-features">{children}</ul>;
}

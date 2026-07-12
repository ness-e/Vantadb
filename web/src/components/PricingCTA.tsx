import { Link } from "@tanstack/react-router";
import { NbBlockAmber } from "./nb";

interface PricingCTAProps {
  heading: string;
  sub: string;
  cta: string;
  href: "/docs" | "/about/contact";
}

export function PricingCTA({ heading, sub, cta, href }: PricingCTAProps) {
  return (
    <NbBlockAmber as="div">
      <div className="nc-price-cta-row">
        <div>
          <h2 className="nc-price-cta-heading">{heading}</h2>
          <p className="nc-price-cta-sub">{sub}</p>
        </div>
        <Link to={href} className="nb-btn nb-btn--ghost nc-price-cta-btn">
          {cta}
        </Link>
      </div>
    </NbBlockAmber>
  );
}

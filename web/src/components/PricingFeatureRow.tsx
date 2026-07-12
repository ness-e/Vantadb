interface PricingFeatureRowProps {
  feature: string;
}

export function PricingFeatureRow({ feature }: PricingFeatureRowProps) {
  return (
    <li className="nc-price-feature">
      {feature}
    </li>
  );
}

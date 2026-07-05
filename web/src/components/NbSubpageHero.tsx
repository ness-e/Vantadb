interface NbSubpageHeroProps {
  pattern?: string;
  title: React.ReactNode;
  sub?: string;
}

export function NbSubpageHero({ pattern = "p01", title, sub }: NbSubpageHeroProps) {
  return (
    <header className={`nb-subpage-hero nb-subpage-hero--${pattern}`} role="banner">
      <div className="nb-subpage-hero-bg" />
      <div className="nb-inner">
        <div className="nb-subpage-hero-body">
          <h1 className="nb-subpage-hero-title">{title}</h1>
          {sub && <p className="nb-subpage-hero-desc">{sub}</p>}
        </div>
      </div>
    </header>
  );
}

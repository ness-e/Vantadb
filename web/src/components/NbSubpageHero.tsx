interface NbSubpageHeroProps {
  num: string;
  title: React.ReactNode;
  sub?: string;
}

export function NbSubpageHero({ num, title, sub }: NbSubpageHeroProps) {
  return (
    <header className="nb-subpage-hero" role="banner">
      <div className="nb-inner">
        <div className="nb-subpage-hero-grid">
          <div className="nb-subpage-hero-meta">
            <span className="nb-subpage-hero-num" aria-hidden="true">
              [{num}]
            </span>
          </div>
          <div>
            <h1 className="nb-subpage-hero-title">{title}</h1>
            {sub && <p className="nb-subpage-hero-desc">{sub}</p>}
          </div>
        </div>
      </div>
    </header>
  );
}

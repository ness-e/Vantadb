import React from "react";

interface NbSubpageHeroProps {
  num: string;
  eyebrow: string;
  title: React.ReactNode;
  sub?: string;
}

const NbSubpageHero: React.FC<NbSubpageHeroProps> = React.memo(
  ({ num, eyebrow, title, sub }) => {
    return (
      <header className="nb-subpage-hero" role="banner">
        <div className="nb-inner">
          <div className="nb-subpage-hero-grid">
            <div className="nb-subpage-hero-meta">
              <span className="nb-subpage-hero-num" aria-hidden="true">
                [{num}]
              </span>
              <span className="nb-subpage-hero-eyebrow">{eyebrow}</span>
            </div>
            <div>
              <h1 className="nb-subpage-hero-title">{title}</h1>
              {sub && <p className="nb-subpage-hero-desc">{sub}</p>}
            </div>
          </div>
        </div>
      </header>
    );
  },
);

export { NbSubpageHero };

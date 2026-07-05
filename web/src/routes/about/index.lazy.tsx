import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import { PendingComponent } from "@/components/PendingComponent";
import "../../styles/about.css";

export const Route = createLazyRoute("/about/")({
  component: AboutIndex,
  pendingComponent: PendingComponent,
});

const STATS = [
  { value: "Apache 2.0", label: "License" },
  { value: "1.2ms", label: "p50 Latency" },
  { value: "Rust", label: "Core Engine" },
  { value: "0.998", label: "Recall@10" },
];

const NAV_SECTIONS = [
  {
    num: "01",
    title: "Company",
    desc: "Who we are, our values, and why we build VantaDB.",
    href: "/about/company",
  },
  {
    num: "02",
    title: "Community",
    desc: "Join the community. Contribute, ask questions, and help shape the future of embedded AI data.",
    href: "/about/community",
  },
  {
    num: "03",
    title: "Contact",
    desc: "Enterprise inquiries, partnerships, or just to say hi.",
    href: "/about/contact",
  },
];

function AboutIndex() {
  const statsRef = useRef<HTMLElement>(null);
  const navRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(statsRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, statsRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(navRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, navRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p19"
        title={
          <span>
            The database that
            <br />
            thinks with you.
          </span>
        }
        sub="We're building the data infrastructure for the AI era — embedded, open-source, and engineered for sub-millisecond performance."
      />

      <NbSection ref={statsRef} ariaLabel="Stats">
        <NbSectionHeader
          monoLabel="[STATS]"
          headline="Facts at a glance."
          sub="What makes VantaDB different, measured."
        />

        <div className="nb-engine-part">
          <div className="nb-grid nb-grid--cols-4">
            {STATS.map((s) => (
              <div key={s.label} className="nb-cell about-stat-card">
                <span className="about-stat-value">{s.value}</span>
                <span className="nb-mono-label">{s.label}</span>
              </div>
            ))}
          </div>
        </div>
      </NbSection>

      <NbSection ref={navRef} className="nb-bg-cross--faint" ariaLabel="Explore">
        <NbSectionHeader
          monoLabel="[EXPLORE]"
          headline="Learn more about VantaDB."
          sub="Company, community, and contact."
        />

        <div className="nb-engine-part">
          <div className="nb-grid nb-grid--cols-3">
            {NAV_SECTIONS.map((s) => (
              <Link key={s.num} to={s.href as "/"} className="nb-cell about-nav-card">
                <span className="nb-mono-label">{s.num}</span>
                <h3 className="nb-card-frame-title">{s.title}</h3>
                <p className="nb-card-frame-desc">{s.desc}</p>
                <span className="about-nav-arrow">{s.href}</span>
              </Link>
            ))}
          </div>
        </div>
      </NbSection>
    </div>
  );
}

import { useRef, memo } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP } from "../lib/gsap";
import { NbSection, NbCopyCommand } from "./nb";
import "../styles/monolith.css";

const CLI_COMMAND = "pip install vantadb-py";

export const NbMonolith = memo(function NbMonolith() {
  const containerRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const tl = gsap.timeline({
          scrollTrigger: {
            trigger: containerRef.current,
            start: "top 75%",
          },
        });

        tl.fromTo(
          ".nb-cta-command",
          { clipPath: "inset(0 0 100% 0)", opacity: 0, y: 30 },
          {
            clipPath: "inset(0)",
            opacity: 1,
            y: 0,
            duration: 0.6,
            ease: "cubic-bezier(0.05, 0.95, 0.3, 1)",
          },
        );

        tl.fromTo(
          ".nb-cta-sub",
          { opacity: 0, y: 12 },
          {
            opacity: 1,
            y: 0,
            duration: 0.35,
            ease: "cubic-bezier(0.05, 0.95, 0.3, 1)",
          },
          "-=0.15",
        );

        tl.fromTo(
          ".nb-meta-tag",
          { opacity: 0, scale: 0.95 },
          {
            opacity: 1,
            scale: 1,
            duration: 0.25,
            stagger: 0.1,
            ease: "cubic-bezier(0.05, 0.95, 0.3, 1)",
          },
          "-=0.1",
        );
      });
    },
    { scope: containerRef },
  );

  return (
    <NbSection ref={containerRef} variant="dark" ariaLabel="Get started">
      <div className="nb-cta-frame">
        <h2 className="nb-cta-headline">SHIP IT.</h2>

        <NbCopyCommand command={CLI_COMMAND} variant="hero" showCopy={true} />

        <p className="nb-cta-sub">Zero servers. One line. Infinite context.</p>

        <div className="nb-cta-actions">
          <Link to="/docs" className="nb-btn nb-btn--ghost nb-btn--ghost-light">
            FULL DOCS
            <span className="nb-cta-arrow">&gt;</span>
          </Link>
        </div>
      </div>

      <div className="nb-meta-row nb-meta-row--centered">
        <span className="nb-meta-tag">ONE BINARY</span>
        <span className="nb-meta-tag">ZERO DEPS</span>
        <span className="nb-meta-tag">MIT LICENSE</span>
      </div>
    </NbSection>
  );
});

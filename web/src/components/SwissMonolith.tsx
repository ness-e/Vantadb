import { useRef, memo } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP } from "../lib/gsap";

export const SwissMonolith = memo(function SwissMonolith() {
  const containerRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        gsap.fromTo(
          ".swiss-monolith-title",
          { clipPath: "inset(0 0 100% 0)", opacity: 0 },
          {
            clipPath: "inset(0)",
            opacity: 1,
            duration: 0.4,
            ease: "cubic-bezier(0.25, 1, 0.5, 1)",
            scrollTrigger: {
              trigger: containerRef.current,
              start: "top 75%",
            },
          },
        );

        gsap.fromTo(
          ".swiss-monolith-subtitle",
          { clipPath: "inset(0 0 100% 0)", opacity: 0 },
          {
            clipPath: "inset(0)",
            opacity: 1,
            duration: 0.35,
            ease: "cubic-bezier(0.25, 1, 0.5, 1)",
            scrollTrigger: {
              trigger: containerRef.current,
              start: "top 75%",
            },
          },
        );
      });
    },
    { scope: containerRef },
  );

  return (
    <section ref={containerRef} className="swiss-monolith-section">
      <div className="swiss-monolith-content">
        <h2 className="swiss-monolith-title">
          pip install vantadb-py
          <span className="monolith-cursor">_</span>
        </h2>

        <p className="swiss-monolith-subtitle">
          Zero servers. One line. Infinite context.
        </p>

        <div className="swiss-monolith-cta-wrap">
          <Link to="/docs" className="swiss-monolith-cta">
            Get Started
          </Link>
        </div>
      </div>
    </section>
  );
});

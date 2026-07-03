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
          ".swiss-monolith-content > *",
          { opacity: 0, y: 12 },
          {
            opacity: 1,
            y: 0,
            duration: 0.5,
            stagger: 0.06,
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

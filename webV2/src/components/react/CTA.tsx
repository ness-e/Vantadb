import React, { useEffect, useRef } from "react";
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);

const CTA: React.FC = () => {
  const sectionRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const ctx = gsap.context(() => {
      gsap.from(".cta-content > *", {
        scrollTrigger: {
          trigger: sectionRef.current,
          start: "top 80%",
        },
        y: 20,
        opacity: 0,
        duration: 0.5,
        stagger: 0.1,
        ease: "power2.out",
      });
    }, sectionRef);

    return () => ctx.revert();
  }, []);

  return (
    <section ref={sectionRef} style={{ background: "var(--amber)", color: "var(--white)" }}>
      <div className="mx-auto px-6 py-24 text-center" style={{ maxWidth: "900px" }}>
        <div className="cta-content">
          <h2 className="font-display font-bold" style={{ fontSize: "var(--text-display)" }}>
            Start building<br />
            with VantaDB
          </h2>

          <p className="mt-4 text-sm opacity-80" style={{ fontFamily: "var(--font-mono)", textTransform: "uppercase", letterSpacing: "0.14em" }}>
            One binary. Three query engines. Zero ops.
          </p>

          <div className="mt-10 flex items-center justify-center gap-4">
            <a
              href="/download"
              className="inline-block px-8 py-3 text-sm font-semibold"
              style={{
                fontFamily: "var(--font-mono)",
                background: "var(--black)",
                color: "var(--white)",
                textTransform: "uppercase",
                letterSpacing: "0.08em",
                textDecoration: "none",
                transition: "opacity 0.15s",
              }}
              onMouseEnter={(e) => (e.currentTarget.style.opacity = "0.85")}
              onMouseLeave={(e) => (e.currentTarget.style.opacity = "1")}
            >
              Download Now
            </a>
            <a
              href="/docs"
              className="inline-block px-8 py-3 text-sm font-semibold"
              style={{
                fontFamily: "var(--font-mono)",
                color: "var(--white)",
                border: "2px solid var(--white)",
                textTransform: "uppercase",
                letterSpacing: "0.08em",
                textDecoration: "none",
                transition: "background 0.15s",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = "rgba(255,255,255,0.1)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = "transparent";
              }}
            >
              Read the Docs &#9658;
            </a>
          </div>
        </div>
      </div>
    </section>
  );
};

export default CTA;

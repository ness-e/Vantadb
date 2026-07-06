import React, { useEffect, useRef } from "react";
import { gsap } from "gsap";

const Hero: React.FC = () => {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const ctx = gsap.context(() => {
      gsap.from(".hero-label", { y: 20, opacity: 0, duration: 0.4, ease: "power2.out" });
      gsap.from(".hero-line", { y: 40, opacity: 0, duration: 0.6, stagger: 0.1, ease: "power2.out" });
      gsap.from(".hero-cta", { y: 20, opacity: 0, duration: 0.5, delay: 0.4, ease: "power2.out" });
    }, containerRef);

    return () => ctx.revert();
  }, []);

  return (
    <section ref={containerRef} className="relative min-h-screen flex items-center justify-center overflow-hidden">
      {/* Background grid pattern */}
      <div className="absolute inset-0 grid-pattern pointer-events-none" />

      <div className="relative z-10 text-center px-6" style={{ maxWidth: "900px" }}>
        <p className="hero-label text-xs mb-6" style={{ fontFamily: "var(--font-mono)", color: "var(--amber)", textTransform: "uppercase", letterSpacing: "0.14em" }}>
          &gt; v1.5.0 — Three query engines
        </p>

        <h1 className="font-display font-bold leading-[0.95] tracking-tight" style={{ fontSize: "var(--text-hero)" }}>
          <span className="hero-line block">The database</span>
          <span className="hero-line block">that thinks</span>
          <span className="hero-line block" style={{ color: "var(--amber)" }}>with you.</span>
        </h1>

        <div className="hero-cta mt-8 flex items-center justify-center gap-4">
          <a
            href="/docs"
            className="inline-block px-6 py-3 text-sm font-semibold"
            style={{ fontFamily: "var(--font-mono)", background: "var(--amber)", color: "var(--white)", textTransform: "uppercase", letterSpacing: "0.08em", textDecoration: "none", transition: "background 0.15s" }}
            onMouseEnter={(e) => (e.currentTarget.style.background = "var(--amber-light)")}
            onMouseLeave={(e) => (e.currentTarget.style.background = "var(--amber)")}
          >
            Get Started &rarr;
          </a>
          <a
            href="https://github.com/vantadb/vantadb"
            className="inline-block px-6 py-3 text-sm font-semibold"
            style={{ fontFamily: "var(--font-mono)", color: "var(--foreground)", border: "1px solid var(--border-strong)", textTransform: "uppercase", letterSpacing: "0.08em", textDecoration: "none", transition: "border-color 0.15s" }}
            onMouseEnter={(e) => (e.currentTarget.style.borderColor = "var(--amber)")}
            onMouseLeave={(e) => (e.currentTarget.style.borderColor = "var(--border-strong)")}
          >
            GitHub &#9658;
          </a>
        </div>
      </div>
    </section>
  );
};

export default Hero;

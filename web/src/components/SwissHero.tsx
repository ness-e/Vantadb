import { useRef } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP } from "../lib/gsap";

export function SwissHero() {
  const containerRef = useRef<HTMLElement>(null);
  const gridLinesRef = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      // 1. Dibujar líneas del grid (scaleY transform origin top)
      gsap.fromTo(
        ".swiss-hero-grid-line",
        { scaleY: 0, transformOrigin: "top" },
        { scaleY: 1, duration: 1.2, stagger: 0.05, ease: "power3.inOut" }
      );

      // 2. Parpadeo táctico de labels
      gsap.fromTo(
        ".swiss-hero-label span",
        { opacity: 0, color: "var(--amber)" },
        {
          opacity: 1,
          color: "var(--amber)",
          duration: 0.1,
          stagger: 0.1,
          repeat: 3,
          yoyo: true,
          onComplete: () => {
            gsap.to(".swiss-hero-label span", { color: "var(--foreground)", duration: 0.1 });
          }
        }
      );

      // 3. Revelado por máscara tipográfica (clip-path style)
      gsap.fromTo(
        ".swiss-hero-title-line",
        { y: "110%", clipPath: "inset(0% 0% 100% 0%)" },
        { 
          y: "0%", 
          clipPath: "inset(0% 0% 0% 0%)", 
          duration: 0.8, 
          stagger: 0.1, 
          ease: "power2.out", 
          delay: 0.4 
        }
      );

      // 4. Aparición de Subtítulo y CTA
      gsap.fromTo(
        [".swiss-hero-subtitle", ".swiss-hero-actions"],
        { opacity: 0, y: 20 },
        { opacity: 1, y: 0, duration: 0.6, stagger: 0.1, ease: "power2.out", delay: 0.8 }
      );
    },
    { scope: containerRef }
  );

  // Generar las 13 líneas para un grid de 12 columnas visible (incluye bordes exteriores)
  const renderGridLines = () => {
    return Array.from({ length: 13 }).map((_, i) => (
      <div key={i} className="swiss-hero-grid-line" style={{ left: `${(i / 12) * 100}%` }} />
    ));
  };

  return (
    <section className="swiss-hero-section" ref={containerRef}>
      {/* Grid visible de fondo */}
      <div className="swiss-hero-grid-overlay" ref={gridLinesRef}>
        {renderGridLines()}
      </div>

      <div className="swiss-grid swiss-hero-layout">
        <div className="swiss-hero-content">
          <div className="swiss-hero-label">
            <span>[RUST-NATIVE]</span> <span>[IN-PROCESS]</span> <span>[ZERO-SERVERS]</span>
          </div>

          <h1 className="swiss-hero-title">
            <span className="swiss-hero-title-wrapper">
              <span className="swiss-hero-title-line">VantaDB</span>
            </span>
          </h1>

          <h2 className="swiss-hero-subtitle">Embedded cognitive memory for AI agents.</h2>

          <div className="swiss-hero-actions">
            <div className="swiss-button-primary">
              <span className="swiss-button-code">pip install vantadb-py</span>
            </div>
            <Link to="/docs" className="swiss-button-ghost">
              Read Docs
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}

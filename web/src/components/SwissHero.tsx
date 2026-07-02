import { useRef, useEffect, useState } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP, TextPlugin } from "../lib/gsap";

export function SwissHero() {
  const containerRef = useRef<HTMLElement>(null);
  const gridLinesRef = useRef<HTMLDivElement>(null);
  const terminalSubtitleRef = useRef<HTMLHeadingElement>(null);

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

      // 4. Efecto Terminal (Typewriter) en el subtítulo
      if (terminalSubtitleRef.current) {
        gsap.to(terminalSubtitleRef.current, {
          duration: 1.5,
          text: "Embedded cognitive memory for AI agents.",
          ease: "none",
          delay: 1.0
        });
      }

      // 5. Aparición de CTA
      gsap.fromTo(
        ".swiss-hero-actions",
        { opacity: 0, y: 20 },
        { opacity: 1, y: 0, duration: 0.6, ease: "power2.out", delay: 1.5 }
      );
    },
    { scope: containerRef }
  );

  // Generar las 13 líneas para un grid de 12 columnas visible (incluye bordes exteriores)
  const renderGridLines = () => {
    return Array.from({ length: 13 }).map((_, i) => (
      <div 
        key={i} 
        className="swiss-hero-grid-line" 
        style={{ 
          position: "absolute",
          top: 0,
          bottom: 0,
          width: "1px",
          background: "var(--border)",
          left: `${(i / 12) * 100}%` 
        }} 
      />
    ));
  };

  return (
    <section 
      className="swiss-hero-section" 
      ref={containerRef}
      style={{ 
        position: "relative",
        minHeight: "85vh",
        display: "flex",
        alignItems: "center",
        background: "var(--background)",
        borderBottom: "1px solid var(--border)",
        overflow: "hidden",
        paddingTop: "64px" // Para evadir el header
      }}
    >
      {/* Grid visible de fondo */}
      <div 
        className="swiss-hero-grid-overlay" 
        ref={gridLinesRef}
        style={{
          position: "absolute",
          inset: 0,
          maxWidth: "1200px",
          margin: "0 auto",
          pointerEvents: "none"
        }}
      >
        {renderGridLines()}
      </div>

      <div className="swiss-grid" style={{ position: "relative", zIndex: 10, width: "100%", padding: "0 24px" }}>
        {/* Contenido en columnas 1-8, dejando 9-12 intencionalmente vacías para asimetría */}
        <div style={{ gridColumn: "1 / 9", display: "flex", flexDirection: "column", gap: "32px", padding: "64px 0" }}>
          
          <div className="swiss-hero-label" style={{ display: "flex", gap: "16px", fontFamily: "var(--font-mono)", fontSize: "var(--text-label)", letterSpacing: "0.14em", fontWeight: 600 }}>
            <span>[RUST-NATIVE]</span> 
            <span>[IN-PROCESS]</span> 
            <span>[ZERO-SERVERS]</span>
          </div>

          <h1 className="swiss-hero-title" style={{ margin: 0, fontFamily: "var(--font-display)", fontSize: "var(--text-hero)", fontWeight: 700, letterSpacing: "-0.05em", lineHeight: 0.95, color: "var(--foreground)" }}>
            <span style={{ display: "block", overflow: "hidden", paddingBottom: "4px" }}>
              <span className="swiss-hero-title-line" style={{ display: "block" }}>VantaDB</span>
            </span>
          </h1>

          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <span style={{ color: "var(--amber)", fontFamily: "var(--font-mono)", fontSize: "var(--text-body)" }}>&gt;</span>
            <h2 
              ref={terminalSubtitleRef}
              style={{ margin: 0, fontFamily: "var(--font-mono)", fontSize: "var(--text-body)", color: "var(--muted)", fontWeight: 400 }}
            >
              {/* Typewriter target */}
            </h2>
            <span style={{ width: "8px", height: "16px", background: "var(--amber)", animation: "blink 1s step-end infinite" }} />
          </div>

          <div className="swiss-hero-actions" style={{ display: "flex", gap: "16px", marginTop: "16px", alignItems: "center" }}>
            <div className="swiss-button-primary" style={{ background: "var(--amber)", color: "#fff", padding: "12px 24px", fontFamily: "var(--font-mono)", fontSize: "0.9rem", fontWeight: 600, cursor: "text" }}>
              pip install vantadb-py
            </div>
            <Link to="/docs" className="swiss-button-ghost" style={{ background: "transparent", border: "1px solid var(--border)", padding: "11px 24px", fontFamily: "var(--font-sans)", fontSize: "0.95rem", color: "var(--foreground)", textDecoration: "none" }}>
              Read Docs
            </Link>
          </div>
        </div>
      </div>
      <style>{`
        @keyframes blink {
          0%, 100% { opacity: 1; }
          50% { opacity: 0; }
        }
      `}</style>
    </section>
  );
}

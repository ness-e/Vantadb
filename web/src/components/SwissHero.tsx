import { useRef, useState } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP } from "../lib/gsap";

export function SwissHero() {
  const containerRef = useRef<HTMLElement>(null);
  const [copied, setCopied] = useState(false);

  useGSAP(
    () => {
      // 1. Dibujar líneas de grid SVG
      gsap.to(".swiss-hero-grid-line", {
        strokeDashoffset: 0,
        duration: 1.2,
        stagger: 0.04,
        ease: "power2.inOut",
      });

      // 2. Flash de los labels (1 solo ciclo táctico)
      gsap.fromTo(
        ".swiss-hero-label span",
        { opacity: 0, color: "var(--amber)" },
        {
          opacity: 1,
          color: "var(--foreground)",
          duration: 0.3,
          stagger: 0.08,
          ease: "power1.in",
        }
      );

      // 3. Revelado por máscara tipográfica del título
      gsap.fromTo(
        ".swiss-hero-title-line",
        { y: "110%", clipPath: "inset(0% 0% 100% 0%)" },
        {
          y: "0%",
          clipPath: "inset(0% 0% 0% 0%)",
          duration: 0.8,
          stagger: 0.1,
          ease: "power3.out",
          delay: 0.4,
        }
      );

      // 4. Fade in del subtítulo en Outfit
      gsap.fromTo(
        ".swiss-hero-subtitle",
        { opacity: 0, y: 15 },
        {
          opacity: 1,
          y: 0,
          duration: 0.6,
          ease: "power2.out",
          delay: 0.8,
        }
      );

      // 5. Aparición de los CTAs
      gsap.fromTo(
        ".swiss-hero-actions",
        { opacity: 0, y: 15 },
        {
          opacity: 1,
          y: 0,
          duration: 0.6,
          ease: "power2.out",
          delay: 1.1,
        }
      );
    },
    { scope: containerRef }
  );

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText("pip install vantadb-py");
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy command: ", err);
    }
  };

  return (
    <section className="swiss-hero-section" ref={containerRef}>
      {/* Grid SVG de Hairlines */}
      <svg 
        className="swiss-hero-grid-svg" 
        viewBox="0 0 1200 800" 
        preserveAspectRatio="none"
        aria-hidden="true"
      >
        {Array.from({ length: 13 }).map((_, i) => {
          const x = (i / 12) * 1200;
          return (
            <line
              key={i}
              x1={x}
              y1="0"
              x2={x}
              y2="800"
              className="swiss-hero-grid-line"
            />
          );
        })}
      </svg>

      <div className="swiss-hero-layout swiss-grid">
        <div className="swiss-hero-content">
          <div className="swiss-hero-label">
            <span>[RUST-NATIVE]</span>
            <span>[IN-PROCESS]</span>
            <span>[ZERO-SERVERS]</span>
          </div>

          <h1 className="swiss-hero-title">
            <span className="swiss-hero-title-wrapper">
              <span className="swiss-hero-title-line">VantaDB</span>
            </span>
          </h1>

          <h2 className="swiss-hero-subtitle">
            <span className="swiss-hero-subtitle-line">
              <span style={{ color: "var(--amber)" }}>&gt;</span>
              <span>Embedded cognitive memory for AI agents</span>
            </span>
          </h2>

          <div className="swiss-hero-actions">
            <button 
              onClick={handleCopy} 
              className="swiss-button-primary"
              aria-label="Copy pip install command"
            >
              <span>{copied ? "Copied!" : "pip install vantadb-py"}</span>
              <svg 
                xmlns="http://www.w3.org/2000/svg" 
                width="14" 
                height="14" 
                viewBox="0 0 24 24" 
                fill="none" 
                stroke="currentColor" 
                strokeWidth="2.5" 
                strokeLinecap="round" 
                strokeLinejoin="round"
                aria-hidden="true"
              >
                {copied ? (
                  <polyline points="20 6 9 17 4 12" />
                ) : (
                  <>
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                  </>
                )}
              </svg>
            </button>
            <Link to="/docs" className="swiss-button-ghost">
              Read Docs
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}


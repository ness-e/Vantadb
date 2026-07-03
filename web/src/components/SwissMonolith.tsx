import { useRef, memo } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP, ScrollTrigger } from "../lib/gsap";

export const SwissMonolith = memo(function SwissMonolith() {
  const containerRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      // Sencilla animación de aparición
      gsap.fromTo(
        ".swiss-monolith-content > *",
        { opacity: 0, y: 30 },
        { 
          opacity: 1, 
          y: 0, 
          duration: 0.8, 
          stagger: 0.1, 
          ease: "power2.out",
          scrollTrigger: {
            trigger: containerRef.current,
            start: "top 75%"
          }
        }
      );
    },
    { scope: containerRef }
  );

  return (
    <section 
      ref={containerRef} 
      style={{ 
        background: "var(--block-dark-bg, #0a0a0a)", 
        color: "var(--block-dark-text, #ffffff)",
        padding: "160px 0",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        textAlign: "center"
      }}
    >
      <div className="swiss-monolith-content" style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "32px", width: "100%", maxWidth: "1200px", padding: "0 24px" }}>
        
        <h2 style={{ 
          margin: 0, 
          fontFamily: "var(--font-display)", 
          fontSize: "var(--text-hero, clamp(3.8rem, 8vw, 7.5rem))", 
          fontWeight: 700, 
          letterSpacing: "-0.05em",
          lineHeight: 1
        }}>
          pip install vantadb-py
        </h2>
        
        <p style={{ 
          margin: 0, 
          fontFamily: "var(--font-sans)", 
          fontSize: "var(--text-title, clamp(1.3rem, 2.2vw, 1.7rem))", 
          color: "var(--block-dark-muted, #808080)",
          letterSpacing: "-0.02em"
        }}>
          Zero servers. One line. Infinite context.
        </p>

        <div style={{ marginTop: "24px" }}>
          <Link 
            to="/docs" 
            style={{ 
              background: "var(--amber)", 
              color: "#ffffff", 
              padding: "16px 32px", 
              fontFamily: "var(--font-sans)",
              fontWeight: 600,
              fontSize: "1.1rem",
              textDecoration: "none",
              display: "inline-block",
              transition: "all 200ms cubic-bezier(0.25, 1, 0.5, 1)",
              willChange: "transform"
            }}
            onMouseEnter={(e) => { e.currentTarget.style.background = "#ffffff"; e.currentTarget.style.color = "#000000"; e.currentTarget.style.transform = "translateY(-2px)"; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = "var(--amber)"; e.currentTarget.style.color = "#ffffff"; e.currentTarget.style.transform = "translateY(0)"; }}
          >
            Get Started
          </Link>
        </div>

      </div>
    </section>
  );
});

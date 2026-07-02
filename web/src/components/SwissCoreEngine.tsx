import { useRef } from "react";
import { gsap, useGSAP, ScrollTrigger } from "../lib/gsap";

const FEATURES = [
  {
    id: "rust",
    title: "Rust Core",
    desc: "Memory safety without garbage collection pauses. Deterministic performance under heavy multi-threaded loads.",
    icon: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
        <rect x="3" y="3" width="18" height="18" />
        <path d="M3 9h18M9 3v18" />
        <circle cx="9" cy="9" r="2" />
      </svg>
    )
  },
  {
    id: "hnsw",
    title: "HNSW Index",
    desc: "Hierarchical Navigable Small World graphs for sub-millisecond high-dimensional vector search.",
    icon: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
        <path d="M4 20L20 4M4 4l16 16M12 4v16" />
        <circle cx="12" cy="12" r="2" />
      </svg>
    )
  },
  {
    id: "bm25",
    title: "BM25 Engine",
    desc: "Full-text search engine built-in. Combine keyword accuracy with semantic recall in a single query.",
    icon: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
        <path d="M4 8h16M4 16h16M8 4v16" />
      </svg>
    )
  },
  {
    id: "wal",
    title: "WAL Durability",
    desc: "Write-Ahead Logging guarantees zero data loss on crashes. Automatic recovery on process restart.",
    icon: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
        <path d="M4 12h16M12 4v16" />
        <rect x="8" y="8" width="8" height="8" />
      </svg>
    )
  },
  {
    id: "pyo3",
    title: "PyO3 Bridge",
    desc: "Native Python bindings with zero IPC overhead. Calls directly into Rust memory space.",
    icon: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
        <path d="M6 6h12v12H6z" />
        <path d="M12 22v-4M12 2v4M2 12h4M22 12h-4" />
      </svg>
    )
  },
  {
    id: "serde",
    title: "Zero-Copy Serde",
    desc: "Data serialization that bypasses the Python GIL. Zero-copy deserialization for massive throughput.",
    icon: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
        <circle cx="12" cy="12" r="8" />
        <path d="M12 4v16M4 12h16" />
      </svg>
    )
  }
];

export function SwissCoreEngine() {
  const sectionRef = useRef<HTMLElement>(null);
  const pinContainerRef = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      const isMobile = window.matchMedia("(max-width: 768px)").matches;
      
      if (!isMobile && pinContainerRef.current) {
        // Pin section and reveal features sequentially
        const features = gsap.utils.toArray(".swiss-ce-feature");
        
        gsap.to(features, {
          y: 0,
          opacity: 1,
          stagger: 0.5,
          ease: "none",
          scrollTrigger: {
            trigger: pinContainerRef.current,
            start: "top top",
            end: "+=150%", // Espacio de scroll para el pin
            pin: true,
            scrub: 1,
          }
        });
      } else {
        // Simple fade in para móvil
        gsap.to(".swiss-ce-feature", {
          y: 0,
          opacity: 1,
          duration: 0.5,
          stagger: 0.1,
          scrollTrigger: {
            trigger: sectionRef.current,
            start: "top 70%",
          }
        });
      }
    },
    { scope: sectionRef }
  );

  return (
    <section 
      ref={sectionRef} 
      style={{ 
        background: "var(--block-dark-bg, #0a0a0a)", 
        color: "var(--block-dark-text, #f0f0f0)",
        position: "relative",
        borderTop: "1px solid var(--border)",
        borderBottom: "1px solid var(--border)",
      }}
    >
      <div 
        ref={pinContainerRef}
        style={{ 
          minHeight: "100vh", 
          display: "flex", 
          flexDirection: "column",
          padding: "80px 0"
        }}
      >
        <div className="swiss-grid" style={{ height: "100%", gap: "0" }}>
          
          {/* Título - span columns 1-12 */}
          <div style={{ gridColumn: "1 / 13", marginBottom: "80px" }}>
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-label)", fontWeight: 600, letterSpacing: "0.14em", color: "var(--block-dark-muted)", textTransform: "uppercase" }}>
              [ARCHITECTURE CORE]
            </span>
            <h2 style={{ fontFamily: "var(--font-display)", fontSize: "var(--text-display)", fontWeight: 700, margin: "24px 0 0 0", letterSpacing: "-0.04em" }}>
              Exploded Architecture.
            </h2>
          </div>

          {/* Grid de features de 3 columnas (dejando el centro para diagrama) */}
          <div style={{ gridColumn: "1 / 13", display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "2px", background: "rgba(255,255,255,0.08)" }}>
            
            {FEATURES.map((feat, i) => (
              <div 
                key={feat.id} 
                className="swiss-ce-feature"
                style={{ 
                  background: "#0a0a0a", 
                  padding: "40px",
                  display: "flex",
                  flexDirection: "column",
                  gap: "24px",
                  opacity: 0,
                  transform: "translateY(20px)"
                }}
              >
                <div style={{ color: "var(--amber)", display: "flex", alignItems: "center", gap: "16px" }}>
                  {feat.icon}
                  <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-label)", color: "var(--block-dark-muted)", letterSpacing: "0.1em" }}>
                    [0{i+1}]
                  </span>
                </div>
                
                <h3 style={{ margin: 0, fontFamily: "var(--font-display)", fontSize: "1.5rem", fontWeight: 600 }}>
                  {feat.title}
                </h3>
                
                <p style={{ margin: 0, fontFamily: "var(--font-sans)", color: "var(--block-dark-muted)", lineHeight: 1.6 }}>
                  {feat.desc}
                </p>
              </div>
            ))}
            
          </div>

        </div>
      </div>
    </section>
  );
}

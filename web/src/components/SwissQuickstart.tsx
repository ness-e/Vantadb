import { useState, useRef, useEffect } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP, TextPlugin, ScrollTrigger } from "../lib/gsap";

const STEPS = [
  {
    num: "01",
    title: "Install",
    cmd: "pip install vantadb-py",
    desc: "Single package. No native dependencies. Works on macOS, Linux, Windows.",
    output: "Successfully installed vantadb-py-0.1.5"
  },
  {
    num: "02",
    title: "Initialize",
    cmd: 'import vantadb_py as vantadb\n\ndb = vantadb.VantaDB("./memory.db")',
    desc: "One import. The database file is created automatically.",
    output: "[VantaDB] Initialized successfully. Embedded engine ready."
  },
  {
    num: "03",
    title: "Store",
    cmd: 'db.put(\n  key="user_42",\n  vector=[0.1, 0.2, 0.3],\n  metadata={"text": "Paris is the capital of France", "source": "wiki"}\n)',
    desc: "Embeddings are generated automatically. No external model needed.",
    output: "[VantaDB] Inserted 1 record. Vector stored."
  },
  {
    num: "04",
    title: "Query",
    cmd: 'results = db.search_memory(\n  query=[0.1, 0.2, 0.3],\n  top_k=5\n)',
    desc: "Semantic + keyword in one call. No orchestration layer.",
    output: "[{\n  'key': 'user_42',\n  'metadata': {'text': 'Paris is the capital of France'},\n  'score': 0.92\n}]"
  },
];

export function SwissQuickstart() {
  const [activeStep, setActiveStep] = useState(0);
  const [hasEntered, setHasEntered] = useState(false);
  const sectionRef = useRef<HTMLElement>(null);
  const codeRef = useRef<HTMLElement>(null);
  const outputRef = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      ScrollTrigger.create({
        trigger: sectionRef.current,
        start: "top 75%",
        onEnter: () => setHasEntered(true),
        once: true
      });
    },
    { scope: sectionRef }
  );

  useEffect(() => {
    if (!hasEntered) return;
    
    if (codeRef.current && outputRef.current) {
      // Ocultar output al principio
      gsap.set(outputRef.current, { opacity: 0 });
      
      const duration = Math.max(0.4, STEPS[activeStep]!.cmd.length * 0.015);
      
      // Matar animaciones previas para evitar conflictos si cambia rápido
      gsap.killTweensOf(codeRef.current);
      
      gsap.to(codeRef.current, {
        duration: duration,
        text: STEPS[activeStep]!.cmd,
        ease: "none",
        onComplete: () => {
          // Mostrar output instantáneamente al terminar de escribir
          gsap.set(outputRef.current, { opacity: 1 });
        }
      });
    }
  }, [activeStep, hasEntered]);

  return (
    <section ref={sectionRef} className="swiss-section" style={{ background: "var(--background)", borderTop: "1px solid var(--border)", paddingTop: "120px", paddingBottom: "120px" }}>
      <div className="swiss-grid" style={{ gap: "24px" }}>
        
        {/* Left: Steps nav (4 columnas) */}
        <div style={{ gridColumn: "1 / 5", display: "flex", flexDirection: "column" }}>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-label)", fontWeight: 600, letterSpacing: "0.14em", color: "var(--steel)", textTransform: "uppercase" }}>
            [QUICKSTART]
          </span>
          <h2 style={{ fontFamily: "var(--font-display)", fontSize: "var(--text-display)", fontWeight: 700, margin: "24px 0 48px 0", letterSpacing: "-0.04em", color: "var(--foreground)" }}>
            Zero to running.
          </h2>

          <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
            {STEPS.map((step, i) => {
              const isActive = activeStep === i;
              return (
                <div
                  key={step.num}
                  onClick={() => setActiveStep(i)}
                  style={{
                    display: "flex",
                    gap: "24px",
                    padding: "16px 0",
                    borderLeft: `2px solid ${isActive ? "var(--amber)" : "transparent"}`,
                    paddingLeft: isActive ? "22px" : "24px",
                    cursor: "pointer",
                    transition: "all 150ms",
                    opacity: isActive ? 1 : 0.5
                  }}
                >
                  <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-label)", fontWeight: 600, color: isActive ? "var(--amber)" : "var(--steel)" }}>
                    [{step.num}]
                  </span>
                  <div>
                    <span style={{ display: "block", fontFamily: "var(--font-mono)", fontSize: "0.9rem", fontWeight: 600, color: isActive ? "var(--foreground)" : "var(--steel)", marginBottom: "8px" }}>
                      {step.title}
                    </span>
                    <p style={{ margin: 0, fontFamily: "var(--font-sans)", fontSize: "0.95rem", color: "var(--muted)", lineHeight: 1.5 }}>
                      {step.desc}
                    </p>
                  </div>
                </div>
              );
            })}
          </div>

          <div style={{ marginTop: "48px" }}>
            <Link to="/docs" className="swiss-button-ghost" style={{ display: "inline-block" }}>
              Read Documentation
            </Link>
          </div>
        </div>

        {/* Right: Code terminal (8 columnas) */}
        <div style={{ gridColumn: "6 / 13" }}>
          <div style={{ 
            background: "#0a0a0a", 
            border: "1px solid rgba(255,255,255,0.08)", 
            borderRadius: "0px",
            height: "100%",
            display: "flex",
            flexDirection: "column"
          }}>
            {/* Terminal Header */}
            <div style={{ 
              display: "flex", 
              alignItems: "center", 
              gap: "12px", 
              padding: "16px 24px", 
              borderBottom: "1px solid var(--border)",
              background: "#111111"
            }}>
              <div style={{ display: "flex", gap: "6px" }}>
                <div style={{ width: "8px", height: "8px", background: "#333333", borderRadius: "50%" }} />
                <div style={{ width: "8px", height: "8px", background: "#333333", borderRadius: "50%" }} />
                <div style={{ width: "8px", height: "8px", background: "#333333", borderRadius: "50%" }} />
              </div>
              <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.75rem", color: "#555555", marginLeft: "auto", textTransform: "uppercase" as const, letterSpacing: "0.1em" }}>
                TERMINAL // PYTHON 3.9+
              </span>
            </div>

            {/* Terminal Body */}
            <div style={{ padding: "32px", flexGrow: 1, display: "flex", flexDirection: "column", gap: "24px" }}>
              <pre style={{ margin: 0, padding: 0, background: "transparent", border: "none" }}>
                <code 
                  ref={codeRef} 
                  style={{ fontFamily: "var(--font-mono)", fontSize: "0.95rem", lineHeight: 1.6, color: "#e0e0e0" }}
                >
                </code>
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.95rem", color: "var(--amber)", animation: "blink 1s step-end infinite", opacity: 0.8 }}>_</span>
              </pre>
              
              <div 
                ref={outputRef} 
                style={{ 
                  marginTop: "auto", 
                  paddingTop: "24px", 
                  borderTop: "1px dashed rgba(255,255,255,0.08)",
                  borderLeft: "2px solid var(--amber)",
                  paddingLeft: "16px"
                }}
              >
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.85rem", color: "#808080", whiteSpace: "pre-wrap" as const, display: "block" }}>
                  {STEPS[activeStep]!.output}
                </span>
              </div>
            </div>
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

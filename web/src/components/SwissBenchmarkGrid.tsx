import { useRef } from "react";
import { gsap, useGSAP, ScrollTrigger } from "../lib/gsap";

const METRICS = [
  {
    id: "latency",
    label: "Query Latency (p50)",
    vanta: "1.2ms",
    traditional: "12ms",
    diff: "↓ 15x faster",
    better: true,
  },
  {
    id: "memory",
    label: "Memory Overhead",
    vanta: "2MB",
    traditional: "180MB",
    diff: "↓ 90x less",
    better: true,
  },
  {
    id: "setup",
    label: "Setup Time",
    vanta: "1 line",
    traditional: "45 min",
    diff: "↓ Instant",
    better: true,
  },
  {
    id: "deps",
    label: "Dependencies",
    vanta: "0",
    traditional: "12+",
    diff: "↓ Zero",
    better: true,
  },
  {
    id: "crash",
    label: "Crash Recovery",
    vanta: "WAL",
    traditional: "Manual",
    diff: "Automatic",
    better: true,
  },
  {
    id: "search",
    label: "Search Type",
    vanta: "Hybrid (BM25+HNSW)",
    traditional: "Single",
    diff: "Full-spectrum",
    better: true,
  },
];

export function SwissBenchmarkGrid() {
  const sectionRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      // Revelar celdas expandiéndose (Bento grid expansion)
      gsap.fromTo(
        ".swiss-vs-cell",
        { opacity: 0, scale: 0.95 },
        {
          opacity: 1,
          scale: 1,
          duration: 0.4,
          stagger: 0.05,
          ease: "power2.out",
          scrollTrigger: {
            trigger: ".swiss-vs-grid",
            start: "top 80%",
          },
        }
      );

      // Count up simple en números
      document.querySelectorAll(".swiss-vs-value").forEach((el) => {
        const targetText = el.getAttribute("data-target") || "";
        const numericMatch = targetText.match(/[\d.]+/);
        if (!numericMatch) return;
        
        const targetNum = parseFloat(numericMatch[0]);
        const isDecimal = targetText.includes(".");
        const prefix = targetText.substring(0, numericMatch.index);
        const suffix = targetText.substring(numericMatch.index! + numericMatch[0].length);

        const obj = { val: 0 };
        gsap.to(obj, {
          val: targetNum,
          duration: 1.2,
          ease: "power3.out",
          scrollTrigger: {
            trigger: el,
            start: "top 85%",
          },
          onUpdate: () => {
            const formatted = isDecimal ? obj.val.toFixed(1) : Math.round(obj.val);
            el.innerHTML = `${prefix}${formatted}${suffix}`;
          },
        });
      });
    },
    { scope: sectionRef }
  );

  return (
    <section className="swiss-section" ref={sectionRef} style={{ background: "var(--background)", paddingTop: "160px" }}>
      <div className="swiss-inner">
        <div className="swiss-vs-header">
          <span className="swiss-eyebrow" style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-label)", fontWeight: 600, letterSpacing: "0.14em" }}>
            [VANTADB] VS [THE STACK]
          </span>
          <h2 className="swiss-vs-title" style={{ fontFamily: "var(--font-display)", fontSize: "var(--text-display)", fontWeight: 700, margin: "24px 0", letterSpacing: "-0.04em" }}>
            Embedded vs. Client/Server.
          </h2>
          <p className="swiss-vs-subtitle" style={{ fontSize: "var(--text-body)", color: "var(--muted)", maxWidth: "600px", marginBottom: "64px" }}>
            By removing the network boundary, VantaDB achieves latencies impossible for traditional
            vector databases while drastically reducing the operational footprint.
          </p>
        </div>

        <div className="swiss-vs-grid" style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(300px, 1fr))", gap: "1px", background: "var(--border)", border: "1px solid var(--border)" }}>
          {METRICS.map((m) => {
            const isNumeric = /\d/.test(m.vanta);
            return (
              <div key={m.id} className="swiss-vs-cell" style={{ background: "var(--background)", padding: "32px", display: "flex", flexDirection: "column", gap: "16px", transition: "all 100ms" }}>
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-label)", color: "var(--steel)", textTransform: "uppercase", letterSpacing: "0.14em" }}>
                  {m.label}
                </span>
                
                <div style={{ display: "flex", alignItems: "baseline", gap: "12px", marginTop: "16px" }}>
                  <span 
                    className="swiss-vs-value"
                    data-target={isNumeric ? m.vanta : undefined}
                    style={{ fontFamily: "var(--font-display)", fontSize: "3.5rem", fontWeight: 700, lineHeight: 0.9, letterSpacing: "-0.04em", color: "var(--foreground)" }}
                  >
                    {isNumeric ? "0" : m.vanta}
                  </span>
                </div>

                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-end", marginTop: "auto", paddingTop: "24px", borderTop: "1px solid var(--border)" }}>
                  <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                    <span style={{ fontSize: "0.75rem", color: "var(--muted)", fontFamily: "var(--font-sans)" }}>Traditional Stack</span>
                    <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.9rem", color: "var(--foreground)" }}>{m.traditional}</span>
                  </div>
                  
                  <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.85rem", fontWeight: 600, color: m.better ? "var(--amber)" : "var(--danger)" }}>
                    {m.diff}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}

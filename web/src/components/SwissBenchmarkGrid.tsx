import { useRef } from "react";
import { gsap, useGSAP } from "../lib/gsap";

const METRICS = [
  {
    id: "latency",
    label: "Query Latency (p50)",
    vanta: "1.2ms",
    traditional: "12ms",
    diff: "↓ 15x faster",
    numericTarget: 1.2,
    suffix: "ms",
    isDecimal: true,
    featured: true,
  },
  {
    id: "memory",
    label: "Memory Overhead",
    vanta: "2MB",
    traditional: "180MB",
    diff: "↓ 90x less",
    numericTarget: 2,
    suffix: "MB",
    isDecimal: false,
    featured: true,
  },
  {
    id: "setup",
    label: "Setup Time",
    vanta: "1 line",
    traditional: "45 min",
    diff: "↓ Instant",
    numericTarget: null,
    suffix: "",
    isDecimal: false,
    featured: false,
  },
  {
    id: "deps",
    label: "Dependencies",
    vanta: "0",
    traditional: "12+",
    diff: "↓ Zero",
    numericTarget: 0,
    suffix: "",
    isDecimal: false,
    featured: false,
  },
  {
    id: "crash",
    label: "Crash Recovery",
    vanta: "WAL",
    traditional: "Manual",
    diff: "Automatic",
    numericTarget: null,
    suffix: "",
    isDecimal: false,
    featured: false,
  },
  {
    id: "search",
    label: "Search Type",
    vanta: "Hybrid (BM25+HNSW)",
    traditional: "Single",
    diff: "Full-spectrum",
    numericTarget: null,
    suffix: "",
    isDecimal: false,
    featured: false,
  },
];

export function SwissBenchmarkGrid() {
  const sectionRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      // Reveal cells with clip-path expansion
      gsap.fromTo(
        ".swiss-vs-cell",
        { opacity: 0, y: 12 },
        {
          opacity: 1,
          y: 0,
          duration: 0.5,
          stagger: 0.06,
          ease: "cubic-bezier(0.25, 1, 0.5, 1)",
          scrollTrigger: {
            trigger: ".swiss-vs-grid",
            start: "top 80%",
          },
        }
      );

      // Count-up ONLY for numeric values
      document.querySelectorAll(".swiss-vs-value[data-numeric]").forEach((el) => {
        const target = parseFloat(el.getAttribute("data-numeric")!);
        const isDecimal = el.getAttribute("data-decimal") === "true";
        const suffix = el.getAttribute("data-suffix") || "";

        const obj = { val: 0 };
        gsap.to(obj, {
          val: target,
          duration: 0.8,
          ease: "power3.out",
          scrollTrigger: {
            trigger: el,
            start: "top 85%",
          },
          onUpdate: () => {
            const formatted = isDecimal ? obj.val.toFixed(1) : Math.round(obj.val);
            el.textContent = `${formatted}${suffix}`;
          },
        });
      });
    },
    { scope: sectionRef }
  );

  return (
    <section
      className="swiss-section"
      ref={sectionRef}
      style={{ background: "var(--background)", paddingTop: "160px" }}
    >
      <div className="swiss-inner">
        {/* Title — NO eyebrow per budget */}
        <h2
          style={{
            fontFamily: "var(--font-display)",
            fontSize: "var(--text-display)",
            fontWeight: 700,
            margin: "0 0 24px 0",
            letterSpacing: "-0.04em",
            color: "var(--foreground)",
          }}
        >
          Embedded vs. Client/Server.
        </h2>
        <p
          style={{
            fontSize: "var(--text-body)",
            color: "var(--muted)",
            maxWidth: "600px",
            marginBottom: "64px",
            lineHeight: 1.65,
          }}
        >
          By removing the network boundary, VantaDB achieves latencies
          impossible for traditional vector databases.
        </p>

        {/* Bento Grid — asymmetric layout */}
        <div
          className="swiss-vs-grid"
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(12, 1fr)",
            gap: "1px",
            background: "var(--border)",
            border: "1px solid var(--border)",
          }}
        >
          {METRICS.map((m) => {
            // Featured metrics span 6 cols, regular span 3 cols (perfect 12-col fit)
            const isFeatured = m.featured;
            const isNumeric = m.numericTarget !== null;

            return (
              <div
                key={m.id}
                className="swiss-vs-cell"
                style={{
                  gridColumn: isFeatured ? "span 6" : "span 3",
                  background: "var(--background)",
                  padding: isFeatured ? "48px 40px" : "32px 24px",
                  display: "flex",
                  flexDirection: "column",
                  gap: "16px",
                  transition: "border-color 100ms",
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = "var(--surface)";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = "var(--background)";
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-label)",
                    color: "var(--steel)",
                    textTransform: "uppercase" as const,
                    letterSpacing: "0.14em",
                  }}
                >
                  {m.label}
                </span>

                <span
                  className="swiss-vs-value"
                  {...(isNumeric
                    ? {
                        "data-numeric": String(m.numericTarget),
                        "data-decimal": String(m.isDecimal),
                        "data-suffix": m.suffix,
                      }
                    : {})}
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: isFeatured ? "4.5rem" : "2.8rem",
                    fontWeight: 700,
                    lineHeight: 0.9,
                    letterSpacing: "-0.04em",
                    color: "var(--foreground)",
                    fontVariantNumeric: "tabular-nums",
                    marginTop: "8px",
                  }}
                >
                  {isNumeric ? "0" : m.vanta}
                </span>

                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "flex-end",
                    marginTop: "auto",
                    paddingTop: "24px",
                    borderTop: "1px solid var(--border)",
                  }}
                >
                  <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                    <span
                      style={{
                        fontSize: "0.75rem",
                        color: "var(--muted)",
                        fontFamily: "var(--font-sans)",
                      }}
                    >
                      Traditional Stack
                    </span>
                    <span
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.9rem",
                        color: "var(--foreground)",
                      }}
                    >
                      {m.traditional}
                    </span>
                  </div>

                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.85rem",
                      fontWeight: 600,
                      color: "var(--amber)",
                    }}
                  >
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

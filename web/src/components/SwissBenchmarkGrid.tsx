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
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
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
          },
        );

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
      });
    },
    { scope: sectionRef },
  );

  return (
    <section
      className="swiss-section swiss-vs-section"
      ref={sectionRef}
    >
      <div className="swiss-inner">
        {/* Title — NO eyebrow per budget */}
        <h2 className="swiss-vs-title">
          Embedded vs. Client/Server.
        </h2>
        <p className="swiss-vs-subtitle">
          By removing the network boundary, VantaDB achieves latencies impossible for traditional
          vector databases.
        </p>

        {/* VS Grid — asymmetric layout */}
        <div className="swiss-vs-grid">
          {METRICS.map((m) => {
            // Featured metrics span 6 cols, regular span 3 cols (perfect 12-col fit)
            const isFeatured = m.featured;
            const isNumeric = m.numericTarget !== null;

            return (
              <div
                key={m.id}
                className={
                  `swiss-vs-cell ` +
                  (isFeatured ? "swiss-vs-cell--featured" : "swiss-vs-cell--regular")
                }
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = "var(--surface)";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = "var(--background)";
                }}
              >
                <span className="swiss-vs-cell-label">
                  {m.label}
                </span>

                <span
                  className={
                    `swiss-vs-value ` +
                    (isFeatured ? "swiss-vs-value--large" : "swiss-vs-value--small")
                  }
                  {...(isNumeric
                    ? {
                        "data-numeric": String(m.numericTarget),
                        "data-decimal": String(m.isDecimal),
                        "data-suffix": m.suffix,
                      }
                    : {})}
                >
                  {isNumeric ? "0" : m.vanta}
                </span>

                <div className="swiss-vs-cell-footer">
                  <div className="swiss-vs-cell-trad">
                    <span className="swiss-vs-cell-trad-label">
                      Traditional Stack
                    </span>
                    <span className="swiss-vs-cell-trad-value">
                      {m.traditional}
                    </span>
                  </div>

                  <span className="swiss-vs-cell-diff">
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

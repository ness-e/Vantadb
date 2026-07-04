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
        const cells = gsap.utils.toArray<HTMLElement>(".swiss-vs-cell");
        if (!cells.length) return;

        const tl = gsap.timeline({
          scrollTrigger: {
            trigger: ".swiss-vs-grid",
            start: "top 80%",
          },
        });

        tl.fromTo(
          cells,
          { scale: 0, opacity: 0, transformOrigin: "center center" },
          {
            scale: 1,
            opacity: 1,
            duration: 0.35,
            stagger: 0.06,
            ease: "cubic-bezier(0.25, 1, 0.5, 1)",
          },
        );

        const labels = document.querySelectorAll<HTMLElement>(".swiss-vs-cell-label");
        tl.fromTo(
          labels,
          { color: "var(--amber)" },
          { color: "var(--steel)", duration: 0.25, stagger: 0.06 },
          "-=0.2",
        );

        cells.forEach((cell) => {
          const mid = cell.dataset.metricId;
          const metric = METRICS.find((m) => m.id === mid);
          if (!metric || metric.numericTarget === null) return;

          const valueEl = cell.querySelector<HTMLElement>("[data-countup]");
          if (!valueEl) return;

          const { numericTarget: target, suffix, isDecimal } = metric;
          const state = { val: 0 };

          tl.to(
            state,
            {
              val: target,
              duration: 0.2,
              ease: "cubic-bezier(0.25, 1, 0.5, 1)",
              onUpdate: () => {
                valueEl.textContent = isDecimal
                  ? `${state.val.toFixed(1)}${suffix}`
                  : `${Math.round(state.val)}${suffix}`;
              },
            },
            0,
          );
        });
      });
    },
    { scope: sectionRef },
  );

  return (
    <section className="swiss-section swiss-vs-section" ref={sectionRef} aria-label="Benchmark comparison">
      <div className="swiss-inner">
        <h2 className="swiss-vs-title">
          Embedded vs. Client/Server.
        </h2>
        <p className="swiss-vs-subtitle">
          By removing the network boundary, VantaDB achieves latencies impossible for traditional
          vector databases.
        </p>

        <div className="swiss-vs-grid" role="list">
          {METRICS.map((m) => {
            const isFeatured = m.featured;
            return (
              <article
                key={m.id}
                data-metric-id={m.id}
                role="listitem"
                className={
                  `swiss-vs-cell ` +
                  (isFeatured ? "swiss-vs-cell--featured" : "swiss-vs-cell--regular")
                }
              >
                <header className="swiss-vs-cell-label">
                  {m.label}
                </header>

                <p
                  data-countup
                  className={
                    `swiss-vs-value ` +
                    (isFeatured ? "swiss-vs-value--large" : "swiss-vs-value--small")
                  }
                  aria-label={`${m.label}: ${m.vanta}`}
                >
                  {m.numericTarget !== null ? "0" : m.vanta}
                </p>

                <footer className="swiss-vs-cell-footer">
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
                </footer>
              </article>
            );
          })}
        </div>
      </div>
    </section>
  );
}

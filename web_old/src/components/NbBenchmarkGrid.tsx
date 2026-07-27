import { useEffect, useRef } from "react";
import { animate, inView } from "motion";
import { useReducedMotion } from "../hooks/useReducedMotion";
import "../styles/benchmark.css";

const METRICS = [
  {
    id: "latency",
    label: "Query Latency (p50)",
    vanta: "1.2ms",
    traditional: "12ms",
    diff: "15x faster",
    numericTarget: 1.2,
    suffix: "ms",
    isDecimal: true,
  },
  {
    id: "memory",
    label: "Memory Overhead",
    vanta: "2MB",
    traditional: "180MB",
    diff: "90x less",
    numericTarget: 2,
    suffix: "MB",
    isDecimal: false,
  },
  {
    id: "setup",
    label: "Setup Time",
    vanta: "1 line",
    traditional: "45 min",
    diff: "Instant",
    numericTarget: null,
    suffix: "",
    isDecimal: false,
  },
  {
    id: "deps",
    label: "Dependencies",
    vanta: "0",
    traditional: "12+",
    diff: "Zero",
    numericTarget: 0,
    suffix: "",
    isDecimal: false,
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
  },
  {
    id: "cost",
    label: "Monthly Cost",
    vanta: "$0",
    traditional: "$2,400+",
    diff: "Free",
    numericTarget: null,
    suffix: "",
    isDecimal: false,
  },
];

export function NbBenchmarkGrid() {
  const sectionRef = useRef<HTMLElement>(null);
  const reducedMotion = useReducedMotion();

  useEffect(() => {
    const el = sectionRef.current;
    if (!el) return;

    if (reducedMotion) return;

    const cleanup = inView(
      el,
      () => {
        const rows = el.querySelectorAll<HTMLElement>(".benchmark-table tbody tr");
        if (rows.length) {
          animate(
            rows,
            { opacity: [0, 1], x: [-24, 0] },
            {
              duration: 0.3,
              delay: 0.04,
              ease: [0.05, 0.95, 0.3, 1],
            },
          );
        }

        METRICS.forEach((m) => {
          if (m.numericTarget === null) return;
          const cell = document.querySelector<HTMLElement>(
            `[data-metric="${m.id}"] [data-countup]`,
          );
          if (!cell) return;

          const state = { val: 0 };
          const { numericTarget: target, suffix, isDecimal } = m;
          animate(
            state,
            {
              val: target,
            },
            {
              duration: 0.2,
              ease: [0.05, 0.95, 0.3, 1],
              onUpdate: () => {
                cell.textContent = isDecimal
                  ? `${state.val.toFixed(1)}${suffix}`
                  : `${Math.round(state.val)}${suffix}`;
              },
            },
          );
        });
      },
      { amount: 0.3 },
    );

    return () => cleanup?.();
  }, [reducedMotion]);

  return (
    <section className="nb-section" ref={sectionRef} aria-label="Benchmark comparison">
      <div className="nb-inner">
        <div>
          <div className="nb-section-header">
            <h2 className="benchmark-title">Embedded vs. Client/Server.</h2>
            <p className="benchmark-subtitle">
              By removing the network boundary, VantaDB achieves latencies impossible for
              traditional vector databases.
            </p>
          </div>

          <div className="benchmark-table-wrap">
            <table className="benchmark-table">
              <thead>
                <tr>
                  <th scope="col">Metric</th>
                  <th scope="col" className="benchmark-col-vanta">
                    VantaDB
                  </th>
                  <th scope="col" className="benchmark-col-trad">
                    Traditional
                  </th>
                  <th scope="col">Gap</th>
                </tr>
              </thead>
              <tbody>
                {METRICS.map((m) => (
                  <tr key={m.id} data-metric={m.id}>
                    <td>
                      <span className="benchmark-metric-label">{m.label}</span>
                    </td>
                    <td className="benchmark-col-vanta">
                      <span data-countup>{m.numericTarget !== null ? "0" : m.vanta}</span>
                    </td>
                    <td className="benchmark-col-trad">{m.traditional}</td>
                    <td className="benchmark-col-diff">{m.diff}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </section>
  );
}

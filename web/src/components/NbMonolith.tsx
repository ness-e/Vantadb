import { Link } from "@tanstack/react-router";
import { memo, useEffect, useRef, useState } from "react";
import { animate, inView } from "motion";
import { NbCopyCommand, NbSection } from "./nb";
import { useReducedMotion } from "../hooks/useReducedMotion";
import "../styles/monolith.css";

const BOOT_MESSAGES = [
  "[OK] Rust core v0.1.5 loaded",
  "[OK] PyO3 bridge: zero-copy FFI active",
  "[OK] HNSW index: 128d, ef_construction=200",
  "[OK] BM25 engine: k1=1.2, b=0.75",
  "[OK] WAL: write-ahead logging enabled",
  "[OK] Hybrid search (RRF) ready",
  "[OK] Memory-mapped storage: ./vantadb.mem",
  "[OK] System ready. Waiting for queries...",
];

const CLI_COMMAND = "pip install vantadb-py";

export const NbMonolith = memo(function NbMonolith() {
  const containerRef = useRef<HTMLElement>(null);
  const progressRef = useRef<HTMLDivElement>(null);
  const [bootIndex, setBootIndex] = useState(0);
  const [bootDone, setBootDone] = useState(false);
  const [progress, setProgress] = useState(0);
  const reducedMotion = useReducedMotion();

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    if (reducedMotion) return;

    const cleanup = inView(
      el,
      () => {
        const cards = [
          { sel: ".nb-boot-title", dur: 0.4, y: 20 },
          { sel: ".nb-boot-install", dur: 0.35, y: 12, delay: 0.2 },
          { sel: ".nb-boot-sub", dur: 0.25, delay: 0.2 },
          { sel: ".nb-boot-actions", dur: 0.25, y: 8, delay: 0.2 },
        ];
        let totalDelay = 0;
        for (const card of cards) {
          const target = el.querySelector(card.sel);
          if (target) {
            totalDelay += card.delay ?? 0;
            animate(
              target,
              { opacity: [0, 1], y: [card.y ?? 12, 0] },
              { duration: card.dur, delay: totalDelay, ease: [0.05, 0.95, 0.3, 1] },
            );
          }
        }
      },
      { amount: 0.3 },
    );

    return () => cleanup?.();
  }, []);

  // Boot sequence timer
  useEffect(() => {
    if (bootDone) return;
    if (bootIndex >= BOOT_MESSAGES.length) {
      setBootDone(true);
      setProgress(100);
      return;
    }
    const t = setTimeout(
      () => {
        setBootIndex((p) => p + 1);
        setProgress(((bootIndex + 1) / BOOT_MESSAGES.length) * 100);
      },
      150 + Math.random() * 200,
    );
    return () => clearTimeout(t);
  }, [bootIndex, bootDone]);

  return (
    <NbSection ref={containerRef} variant="dark" ariaLabel="Get started">
      <div className="nb-boot">
        <pre className="nb-boot-ascii" aria-hidden="true">
          {`╔══════════════════════════════════╗
║          V A N T A D B           ║
║  Embedded Memory Engine v0.1.5   ║
╚══════════════════════════════════╝`}
        </pre>

        <h2 className="nb-boot-title">Deploy in one line.</h2>

        <NbCopyCommand
          command={CLI_COMMAND}
          variant="hero"
          showCopy={true}
          className="nb-boot-install"
        />

        <p className="nb-boot-sub">Zero servers. One line. Infinite context.</p>

        <div className="nb-boot-progress-wrap">
          <div className="nb-boot-progress-label">
            <span>BOOT SEQUENCE</span>
            <span>{bootDone ? "COMPLETE" : `${Math.round(progress)}%`}</span>
          </div>
          <div className="nb-boot-progress-track">
            <div
              ref={progressRef}
              className="nb-boot-progress-fill"
              style={{ width: `${progress}%` }}
            />
          </div>
          <div className="nb-boot-progress-msgs">
            {BOOT_MESSAGES.slice(0, bootIndex).map((msg, i) => (
              <span key={i} className="nb-boot-msg">
                {msg}
              </span>
            ))}
            {bootDone && (
              <span className="nb-boot-msg nb-boot-msg--ready">
                [READY] awaiting your command...
              </span>
            )}
          </div>
        </div>

        <div className="nb-boot-actions">
          <Link to="/docs" className="nb-btn nb-btn--ghost nb-btn--ghost-light">
            FULL DOCS <span className="nb-boot-arrow">&gt;</span>
          </Link>
        </div>
      </div>

      <div className="nb-meta-row nb-meta-row--centered">
        <span className="nb-meta-tag">ONE BINARY</span>
        <span className="nb-meta-tag">ZERO DEPS</span>
        <span className="nb-meta-tag">MIT LICENSE</span>
      </div>
    </NbSection>
  );
});

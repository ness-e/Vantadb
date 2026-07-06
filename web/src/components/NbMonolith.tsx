import { useRef, useState, useEffect, memo } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP } from "../lib/gsap";
import { NbSection, NbCopyCommand } from "./nb";
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

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const tl = gsap.timeline({
          scrollTrigger: { trigger: containerRef.current, start: "top 75%" },
        });
        tl.fromTo(
          ".nb-boot-title",
          { opacity: 0, y: 20 },
          { opacity: 1, y: 0, duration: 0.4, ease: "var(--ease-swiss)" },
        );
        tl.fromTo(
          ".nb-boot-install",
          { opacity: 0, y: 12 },
          { opacity: 1, y: 0, duration: 0.35 },
          "-=0.15",
        );
        tl.fromTo(".nb-boot-sub", { opacity: 0 }, { opacity: 1, duration: 0.25 }, "-=0.1");
        tl.fromTo(
          ".nb-boot-actions",
          { opacity: 0, y: 8 },
          { opacity: 1, y: 0, duration: 0.25 },
          "-=0.05",
        );
      });
    },
    { scope: containerRef },
  );

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
        {/* ASCII Logo */}
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

        {/* Boot progress */}
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

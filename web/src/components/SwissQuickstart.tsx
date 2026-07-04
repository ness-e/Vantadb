import { useState, useRef, useEffect, useCallback } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP, TextPlugin, ScrollTrigger } from "../lib/gsap";

const STEPS = [
  {
    num: "01",
    title: "Install",
    cmd: "pip install vantadb-py",
    desc: "Single package. No native dependencies. Works on macOS, Linux, Windows.",
    output: "Successfully installed vantadb-py-0.1.5",
  },
  {
    num: "02",
    title: "Initialize",
    cmd: 'import vantadb_py as vantadb\n\ndb = vantadb.VantaDB("./memory.db")',
    desc: "One import. The database file is created automatically.",
    output: "[VantaDB] Initialized successfully. Embedded engine ready.",
  },
  {
    num: "03",
    title: "Store",
    cmd: 'db.put(\n  namespace="agent/main",\n  key="user_42",\n  payload="Paris is the capital of France",\n  metadata={"source": "wiki"},\n  vector=[0.1, 0.2, 0.3]\n)',
    desc: "Schema-free. Store text payloads with optional metadata and vectors.",
    output: "[VantaDB] Inserted 1 record. Vector stored.",
  },
  {
    num: "04",
    title: "Query",
    cmd: "results = db.search_memory(\n  namespace=\"agent/main\",\n  query_vector=[0.1, 0.2, 0.3],\n  top_k=5\n)",
    desc: "Semantic + keyword in one call. No orchestration layer.",
    output:
      "{\n  'records': [{'record': {'key': 'user_42', 'payload': 'Paris is the capital of France'}, 'score': 0.92}]\n}",
  },
];

export function SwissQuickstart() {
  const [activeStep, setActiveStep] = useState(0);
  const [hasEntered, setHasEntered] = useState(false);
  const sectionRef = useRef<HTMLElement>(null);
  const codeRef = useRef<HTMLElement>(null);
  const outputRef = useRef<HTMLDivElement>(null);
  const loopRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        ScrollTrigger.create({
          trigger: sectionRef.current,
          start: "top 75%",
          onEnter: () => setHasEntered(true),
          once: true,
        });
      });
    },
    { scope: sectionRef },
  );

  const typeStep = useCallback(
    (stepIndex: number, onComplete: () => void) => {
      const step = STEPS[stepIndex];
      if (!step) { onComplete(); return; }

      setActiveStep(stepIndex);

      if (codeRef.current && outputRef.current) {
        gsap.set(outputRef.current, { opacity: 0 });
        gsap.killTweensOf(codeRef.current);

        const duration = Math.max(0.25, step.cmd.length * 0.03);

        gsap.to(codeRef.current, {
          duration,
          text: step.cmd,
          ease: "none",
          onComplete: () => {
            gsap.set(outputRef.current, { opacity: 1 });
            onComplete();
          },
        });
      } else {
        onComplete();
      }
    },
    [],
  );

  useEffect(() => {
    if (!hasEntered) return;

    let cancelled = false;

    const runLoop = () => {
      if (cancelled) return;

      let i = 0;
      const playNext = () => {
        if (cancelled) return;
        if (i >= STEPS.length) {
          loopRef.current = setTimeout(runLoop, 3000);
          return;
        }
        typeStep(i, () => {
          i++;
          setTimeout(playNext, 400);
        });
      };
      playNext();
    };

    runLoop();

    return () => {
      cancelled = true;
      if (loopRef.current) clearTimeout(loopRef.current);
    };
  }, [hasEntered, typeStep]);

  return (
    <section ref={sectionRef} className="swiss-section qs-section" aria-label="Quickstart guide">
      <div className="swiss-grid qs-grid">
        <div className="quickstart-left">
          <h2 className="qs-heading">
            Zero to running.
          </h2>

          <nav className="qs-steps" aria-label="Setup steps">
            {STEPS.map((step, i) => {
              const isActive = activeStep === i;
              return (
                <button
                  key={step.num}
                  onClick={() => setActiveStep(i)}
                  className={`qs-step ${isActive ? "qs-step--active" : ""}`}
                  aria-current={isActive ? "step" : undefined}
                  aria-label={`Step ${step.num}: ${step.title}`}
                >
                  <span className="qs-step-num" aria-hidden="true">
                    [{step.num}]
                  </span>
                  <div className="qs-step-body">
                    <span className="qs-step-title">
                      {step.title}
                    </span>
                    <p className="qs-step-desc">
                      {step.desc}
                    </p>
                  </div>
                </button>
              );
            })}
          </nav>

          <div className="qs-docs">
            <Link to="/docs" className="btn-ghost btn-ghost--hero qs-docs-link" aria-label="Read documentation">
              Read Documentation
            </Link>
          </div>
        </div>

        <div className="quickstart-right">
          <div className="qs-terminal" role="region" aria-label="Terminal preview">
            <header className="qs-terminal-header">
              <div className="qs-terminal-dots" aria-hidden="true">
                <div className="qs-terminal-dot" />
                <div className="qs-terminal-dot" />
                <div className="qs-terminal-dot" />
              </div>
              <span className="qs-terminal-label">
                TERMINAL // PYTHON 3.9+
              </span>
            </header>

            <div className="qs-terminal-body">
              <pre className="qs-code-pre">
                <code ref={codeRef} className="qs-code" aria-live="polite"></code>
                <span className="qs-cursor" aria-hidden="true">_</span>
              </pre>

              <div ref={outputRef} className="qs-output" aria-live="polite">
                <span className="qs-output-text">
                  {STEPS[activeStep]!.output}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

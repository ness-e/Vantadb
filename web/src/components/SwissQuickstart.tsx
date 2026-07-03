import { useState, useRef, useEffect } from "react";
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

  useEffect(() => {
    if (!hasEntered) return;

    const mm = gsap.matchMedia();
    mm.add("(prefers-reduced-motion: no-preference)", () => {
      if (codeRef.current && outputRef.current) {
        gsap.set(outputRef.current, { opacity: 0 });

        const duration = Math.max(0.4, STEPS[activeStep]!.cmd.length * 0.015);

        gsap.killTweensOf(codeRef.current);

        gsap.to(codeRef.current, {
          duration: duration,
          text: STEPS[activeStep]!.cmd,
          ease: "none",
          onComplete: () => {
            gsap.set(outputRef.current, { opacity: 1 });
          },
        });
      }
    });

    return () => {
      mm.revert();
    };
  }, [activeStep, hasEntered]);

  return (
    <section
      ref={sectionRef}
      className="swiss-section qs-section"
    >
      <div className="swiss-grid qs-grid">
        {/* Left: Steps nav (4 columnas en desktop, 12 en mobile/tablet) */}
        <div className="quickstart-left">
          <h2 className="qs-heading">
            Zero to running.
          </h2>

          <div className="qs-steps">
            {STEPS.map((step, i) => {
              const isActive = activeStep === i;
              return (
                <div
                  key={step.num}
                  onClick={() => setActiveStep(i)}
                  className={`qs-step ${isActive ? "qs-step--active" : ""}`}
                >
                  <span className="qs-step-num">
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
                </div>
              );
            })}
          </div>

          <div className="qs-docs">
            <Link to="/docs" className="btn-ghost btn-ghost--hero qs-docs-link">
              Read Documentation
            </Link>
          </div>
        </div>

        {/* Right: Code terminal (8 columnas en desktop, 12 en mobile/tablet) */}
        <div className="quickstart-right">
          <div className="qs-terminal">
            {/* Terminal Header */}
            <div className="qs-terminal-header">
              <div className="qs-terminal-dots">
                <div className="qs-terminal-dot" />
                <div className="qs-terminal-dot" />
                <div className="qs-terminal-dot" />
              </div>
              <span className="qs-terminal-label">
                TERMINAL // PYTHON 3.9+
              </span>
            </div>

            {/* Terminal Body */}
            <div className="qs-terminal-body">
              <pre className="qs-code-pre">
                <code
                  ref={codeRef}
                  className="qs-code"
                ></code>
                <span className="qs-cursor">
                  _
                </span>
              </pre>

              <div
                ref={outputRef}
                className="qs-output"
              >
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

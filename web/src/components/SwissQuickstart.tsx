import { useState, useRef, useEffect, useCallback } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP, TextPlugin, ScrollTrigger } from "../lib/gsap";
import "../styles/quickstart.css";

const STEPS = [
  {
    num: "01",
    title: "Install",
    cmd: "pip install vantadb-py",
    desc: "Single package. No native dependencies.",
    output: "Successfully installed vantadb-py-0.1.5",
  },
  {
    num: "02",
    title: "Initialize",
    cmd: 'import vantadb_py as vantadb\n\ndb = vantadb.VantaDB("./memory.db")',
    desc: "One import. Database file created automatically.",
    output: "[VantaDB] Initialized successfully. Embedded engine ready.",
  },
  {
    num: "03",
    title: "Store",
    cmd: 'db.put(\n  namespace="agent/main",\n  key="user_42",\n  payload="Paris is the capital of France",\n  metadata={"source": "wiki"},\n  vector=[0.1, 0.2, 0.3]\n)',
    desc: "Schema-free. Store text payloads with metadata and vectors.",
    output: "[VantaDB] Inserted 1 record. Vector stored.",
  },
  {
    num: "04",
    title: "Query",
    cmd: 'results = db.search_memory(\n  namespace="agent/main",\n  query_vector=[0.1, 0.2, 0.3],\n  top_k=5\n)',
    desc: "Semantic + keyword in one call.",
    output: "{\n  'records': [{'record': {'key': 'user_42', 'payload': 'Paris is the capital of France'}, 'score': 0.92}]\n}",
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

  const typeStep = useCallback((stepIndex: number, onComplete: () => void) => {
    const step = STEPS[stepIndex];
    if (!step) {
      onComplete();
      return;
    }

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
  }, []);

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
    <section ref={sectionRef} className="nb-section nb-section--lg" aria-label="Quickstart guide">
      <div className="nb-inner nb-split-5-7">
        <div>
          <div className="nb-section-header nb-section-header--bordered">
            <span className="nb-label nb-label--amber">&gt; quickstart</span>
            <h2 className="nb-crt-heading">Zero to running.</h2>
          </div>

          <nav className="nb-crt-steps" aria-label="Setup steps">
            {STEPS.map((step, i) => {
              const isActive = activeStep === i;
              return (
                <button
                  key={step.num}
                  onClick={() => {
                    setActiveStep(i);
                    if (codeRef.current) {
                      gsap.killTweensOf(codeRef.current);
                      gsap.set(codeRef.current, { text: step.cmd });
                    }
                    if (outputRef.current) {
                      gsap.set(outputRef.current, { opacity: 1 });
                    }
                  }}
                  className={`nb-crt-step ${isActive ? "nb-crt-step--active" : ""}`}
                  aria-current={isActive ? "step" : undefined}
                  aria-label={`Step ${step.num}: ${step.title}`}
                >
                  <span className="nb-crt-step-prefix" aria-hidden="true">&gt;</span>
                  <div className="nb-crt-step-body">
                    <span className="nb-crt-step-title nb-label">
                      [{step.num}] {step.title}
                    </span>
                    <p className="nb-crt-step-desc">{step.desc}</p>
                  </div>
                </button>
              );
            })}
          </nav>

          <div className="nb-crt-cta">
            <Link
              to="/docs"
              className="nb-arrow"
              aria-label="Read documentation"
            >
              Read Documentation
            </Link>
          </div>
        </div>

        <div>
          <div className="nb-crt-terminal" role="region" aria-label="Terminal preview">
            <header className="nb-crt-terminal-bar">
              <span className="nb-crt-terminal-label">[ QUICKSTART ]</span>
            </header>

            <div className="nb-crt-terminal-body">
              <pre className="nb-crt-code-pre">
                <code ref={codeRef} className="nb-crt-code" aria-live="polite" />
                <span className="nb-crt-cursor" aria-hidden="true">_</span>
              </pre>

              <div ref={outputRef} className="nb-crt-output" aria-live="polite">
                <span className="nb-crt-output-text">{STEPS[activeStep]!.output}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
